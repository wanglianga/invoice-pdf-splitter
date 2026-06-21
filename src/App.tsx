import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import type {
  InvoiceInfo,
  ProcessedInvoice,
  ProcessingProgress,
  AppSettings,
  LogEntry,
  PageClassification,
  NamingPreview,
} from "./types";
import DropZone from "./components/DropZone";
import SettingsPanel from "./components/SettingsPanel";
import PreviewPanel from "./components/PreviewPanel";
import LogPanel from "./components/LogPanel";
import PageClassificationPanel from "./components/PageClassificationPanel";

const defaultSettings: AppSettings = {
  namingTemplate: "{日期}_{销售方}_{发票号码}_{金额}",
  outputDirectory: "",
  duplicateHandling: "rename",
  manualCorrection: false,
  enableLog: true,
};

const normalizePageClassification = (data: any): PageClassification => ({
  pageNumber: data.page_number ?? data.pageNumber ?? 0,
  originalFile: data.original_file ?? data.originalFile ?? "",
  pageType: data.page_type ?? data.pageType ?? "未知",
  pageAction: data.page_action ?? data.pageAction ?? "manual",
  confidence: data.confidence ?? 0,
  info: normalizeInvoiceInfo(data.info),
});

const normalizeInvoiceInfo = (data: any): InvoiceInfo => ({
  invoiceNumber: data.invoice_number ?? data.invoiceNumber ?? "未识别",
  buyer: data.buyer ?? "未识别",
  seller: data.seller ?? "未识别",
  amount: data.amount ?? "0.00",
  date: data.date ?? "未知日期",
  invoiceType: data.invoice_type ?? data.invoiceType ?? "普通发票",
  isReimbursement: data.is_reimbursement ?? data.isReimbursement ?? false,
  pageNumber: data.page_number ?? data.pageNumber ?? 0,
  originalFile: data.original_file ?? data.originalFile ?? "",
  reimburser: data.reimburser ?? "未识别",
  projectCode: data.project_code ?? data.projectCode ?? "未识别",
  pageType: data.page_type ?? data.pageType ?? "",
  pageAction: data.page_action ?? data.pageAction ?? "",
});

const normalizeNamingPreview = (data: any): NamingPreview => ({
  template: data.template ?? "",
  fileName: data.file_name ?? data.fileName ?? "",
  warnings: (data.warnings ?? []).map((w: any) => ({
    warningType: w.warning_type ?? w.warningType ?? "",
    message: w.message ?? "",
    field: w.field ?? undefined,
  })),
});

export default function App() {
  const [files, setFiles] = useState<string[]>([]);
  const [settings, setSettings] = useState<AppSettings>(defaultSettings);
  const [invoices, setInvoices] = useState<ProcessedInvoice[]>([]);
  const [classifications, setClassifications] = useState<PageClassification[]>([]);
  const [namingPreview, setNamingPreview] = useState<NamingPreview[]>([]);
  const [workflowStage, setWorkflowStage] = useState<"idle" | "classified" | "processing" | "completed">("idle");

  const normalizeInvoice = (data: any): ProcessedInvoice => {
    if (data.info) {
      return {
        invoiceNumber: data.info.invoice_number || "未识别",
        buyer: data.info.buyer || "未识别",
        seller: data.info.seller || "未识别",
        amount: data.info.amount || "0.00",
        date: data.info.date || "未知日期",
        invoiceType: data.info.invoice_type || "普通发票",
        isReimbursement: data.info.is_reimbursement || false,
        pageNumber: data.info.page_number || 0,
        originalFile: data.info.original_file || "",
        reimburser: data.info.reimburser || "未识别",
        projectCode: data.info.project_code || "未识别",
        pageType: data.info.page_type || "",
        pageAction: data.info.page_action || "",
        outputFileName: data.output_file_name || "",
        outputPath: data.output_path || "",
        success: data.success || false,
        errorMessage: data.error_message || undefined,
      };
    }
    return data as ProcessedInvoice;
  };

  const [progress, setProgress] = useState<ProcessingProgress>({
    currentFile: "",
    currentPage: 0,
    totalPages: 0,
    processed: 0,
    total: 0,
    status: "idle",
  });
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [activeTab, setActiveTab] = useState<"preview" | "logs" | "classify">("classify");
  const [correctingInvoice, setCorrectingInvoice] = useState<ProcessedInvoice | null>(null);
  const logRef = useRef<HTMLDivElement>(null);

  const addLog = useCallback((level: LogEntry["level"], message: string) => {
    const entry: LogEntry = {
      timestamp: new Date().toLocaleTimeString(),
      level,
      message,
    };
    setLogs((prev) => [...prev, entry]);
  }, []);

  useEffect(() => {
    const unlistenProgress = listen("processing-progress", (event) => {
      const data = event.payload as ProcessingProgress;
      setProgress(data);
    });

    const unlistenInvoice = listen("invoice-processed", (event) => {
      const data = normalizeInvoice(event.payload);
      setInvoices((prev) => [...prev, data]);
      if (data.success) {
        addLog("success", `✓ ${data.outputFileName} (第${data.pageNumber}页)`);
      } else {
        addLog("error", `✗ 第${data.pageNumber}页处理失败: ${data.errorMessage}`);
      }
    });

    const unlistenLog = listen("processing-log", (event) => {
      const data = event.payload as { level: string; message: string };
      addLog(data.level as LogEntry["level"], data.message);
    });

    return () => {
      unlistenProgress.then((f) => f());
      unlistenInvoice.then((f) => f());
      unlistenLog.then((f) => f());
    };
  }, [addLog]);

  useEffect(() => {
    if (logRef.current) {
      logRef.current.scrollTop = logRef.current.scrollHeight;
    }
  }, [logs]);

  const refreshNamingPreview = useCallback(async (currentClassifications: PageClassification[], template: string) => {
    if (currentClassifications.length === 0) {
      setNamingPreview([]);
      return;
    }
    try {
      const previews = await invoke<NamingPreview[]>("preview_file_names", {
        namingTemplate: template,
        pages: currentClassifications.map((c) => ({
          page_number: c.pageNumber,
          original_file: c.originalFile,
          page_type: c.pageType,
          page_action: c.pageAction,
          confidence: c.confidence,
          info: {
            invoice_number: c.info.invoiceNumber,
            buyer: c.info.buyer,
            seller: c.info.seller,
            amount: c.info.amount,
            date: c.info.date,
            invoice_type: c.info.invoiceType,
            is_reimbursement: c.info.isReimbursement,
            page_number: c.info.pageNumber,
            original_file: c.info.originalFile,
            reimburser: c.info.reimburser,
            project_code: c.info.projectCode,
            page_type: c.info.pageType,
            page_action: c.info.pageAction,
          },
        })),
      });
      setNamingPreview(previews.map(normalizeNamingPreview));
    } catch {
      setNamingPreview([]);
    }
  }, []);

  useEffect(() => {
    if (classifications.length > 0) {
      refreshNamingPreview(classifications, settings.namingTemplate);
    }
  }, [classifications, settings.namingTemplate, refreshNamingPreview]);

  const handleFilesAdded = useCallback((newFiles: string[]) => {
    setFiles((prev) => {
      const combined = [...prev, ...newFiles];
      return [...new Set(combined)];
    });
    addLog("info", `添加了 ${newFiles.length} 个文件/文件夹`);
  }, [addLog]);

  const handleRemoveFile = useCallback((file: string) => {
    setFiles((prev) => prev.filter((f) => f !== file));
  }, []);

  const handleSelectOutputDir = useCallback(async () => {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "选择输出目录",
    });
    if (selected && typeof selected === "string") {
      setSettings((prev) => ({ ...prev, outputDirectory: selected }));
    }
  }, []);

  const handleClassifyPages = useCallback(async () => {
    if (files.length === 0) {
      addLog("warn", "请先添加 PDF 文件");
      return;
    }

    addLog("info", "开始识别页面类型...");
    setInvoices([]);
    setClassifications([]);

    try {
      const allClassifications: PageClassification[] = [];

      for (const file of files) {
        const result = await invoke<PageClassification[]>("classify_pdf_pages", {
          pdfPath: file,
        });
        const normalized = result.map(normalizePageClassification);
        allClassifications.push(...normalized);
      }

      setClassifications(allClassifications);
      setWorkflowStage("classified");
      setActiveTab("classify");
      addLog("success", `页面识别完成，共 ${allClassifications.length} 页`);

      const typeCount: Record<string, number> = {};
      for (const c of allClassifications) {
        typeCount[c.pageType] = (typeCount[c.pageType] || 0) + 1;
      }
      for (const [type, count] of Object.entries(typeCount)) {
        addLog("info", `  ${type}: ${count} 页`);
      }
    } catch (error) {
      addLog("error", `页面识别失败: ${error}`);
    }
  }, [files, addLog]);

  const handleConfirmBatchSplit = useCallback(async (confirmedPages: PageClassification[]) => {
    if (!settings.outputDirectory) {
      addLog("warn", "请先选择输出目录");
      return;
    }

    setWorkflowStage("processing");
    setInvoices([]);
    setProgress((prev) => ({ ...prev, status: "processing", processed: 0 }));
    addLog("info", "开始批量拆分...");

    try {
      for (const file of files) {
        const filePages = confirmedPages.filter((p) => p.originalFile === file.split(/[/\\]/).pop());

        if (filePages.length === 0) continue;

        const pagesForBackend = filePages.map((p) => ({
          page_number: p.pageNumber,
          original_file: p.originalFile,
          page_type: p.pageType,
          page_action: p.pageAction,
          confidence: p.confidence,
          info: {
            invoice_number: p.info.invoiceNumber,
            buyer: p.info.buyer,
            seller: p.info.seller,
            amount: p.info.amount,
            date: p.info.date,
            invoice_type: p.info.invoiceType,
            is_reimbursement: p.info.isReimbursement,
            page_number: p.info.pageNumber,
            original_file: p.info.originalFile,
            reimburser: p.info.reimburser,
            project_code: p.info.projectCode,
            page_type: p.info.pageType,
            page_action: p.info.pageAction,
          },
        }));

        const results = await invoke<ProcessedInvoice[]>("batch_split_confirmed", {
          pdfPath: file,
          outputDir: settings.outputDirectory,
          namingTemplate: settings.namingTemplate,
          duplicateHandling: settings.duplicateHandling,
          confirmedPages: pagesForBackend,
        });

        const normalized = results.map(normalizeInvoice);
        setInvoices((prev) => [...prev, ...normalized]);
      }

      setWorkflowStage("completed");
      setProgress((prev) => ({ ...prev, status: "completed" }));
      setActiveTab("preview");
      addLog("success", "批量拆分完成！");
    } catch (error) {
      setWorkflowStage("classified");
      setProgress((prev) => ({ ...prev, status: "failed" }));
      addLog("error", `批量拆分失败: ${error}`);
    }
  }, [files, settings, addLog]);

  const handleStartProcessing = useCallback(async () => {
    if (files.length === 0) {
      addLog("warn", "请先添加 PDF 文件");
      return;
    }
    if (!settings.outputDirectory) {
      addLog("warn", "请先选择输出目录");
      return;
    }

    setInvoices([]);
    setProgress((prev) => ({ ...prev, status: "processing", processed: 0 }));
    setWorkflowStage("processing");
    addLog("info", "开始处理发票...");

    try {
      await invoke("process_invoices", {
        files,
        settings: {
          naming_template: settings.namingTemplate,
          output_directory: settings.outputDirectory,
          duplicate_handling: settings.duplicateHandling,
          manual_correction: settings.manualCorrection,
          enable_log: settings.enableLog,
        },
      });
      setProgress((prev) => ({ ...prev, status: "completed" }));
      setWorkflowStage("completed");
      addLog("success", "处理完成！");
    } catch (error) {
      setProgress((prev) => ({ ...prev, status: "failed" }));
      setWorkflowStage("idle");
      addLog("error", `处理失败: ${error}`);
    }
  }, [files, settings, addLog]);

  const handleCorrectInvoice = useCallback((invoice: ProcessedInvoice) => {
    setCorrectingInvoice(invoice);
  }, []);

  const handleSaveCorrection = useCallback((corrected: InvoiceInfo) => {
    setInvoices((prev) =>
      prev.map((inv) =>
        inv.pageNumber === corrected.pageNumber && inv.originalFile === corrected.originalFile
          ? { ...inv, ...corrected }
          : inv
      )
    );
    setCorrectingInvoice(null);
    addLog("info", `已修正第 ${corrected.pageNumber} 页发票信息`);
  }, [addLog]);

  const successCount = invoices.filter((i) => i.success).length;
  const failedCount = invoices.filter((i) => !i.success).length;

  return (
    <div className="app-container">
      <header className="app-header">
        <div>
          <h1>📄 发票 PDF 拆分命名工具</h1>
          <div className="subtitle">本地处理 · 数据安全 · 智能识别 · 多类型分类</div>
        </div>
        <div style={{ fontSize: 12, opacity: 0.8 }}>
          已添加 {files.length} 个文件
          {workflowStage !== "idle" && ` · ${workflowStage === "classified" ? "已识别" : workflowStage === "processing" ? "处理中" : "已完成"}`}
        </div>
      </header>

      <div className="main-content">
        <aside className="sidebar">
          <div className="sidebar-section">
            <h3>📁 添加文件</h3>
            <DropZone onFilesAdded={handleFilesAdded} />
            <div className="file-list">
              {files.map((file) => (
                <div key={file} className="file-item">
                  <span className="file-name" title={file}>
                    {file.split(/[/\\]/).pop()}
                  </span>
                  <span className="remove-btn" onClick={() => handleRemoveFile(file)}>
                    ×
                  </span>
                </div>
              ))}
            </div>
          </div>

          <SettingsPanel
            settings={settings}
            onSettingsChange={setSettings}
            onSelectOutputDir={handleSelectOutputDir}
            namingPreview={namingPreview}
            classifications={classifications}
          />

          <div className="sidebar-section">
            {workflowStage === "idle" || workflowStage === "completed" ? (
              <>
                <button
                  className="btn btn-primary"
                  onClick={handleClassifyPages}
                  disabled={files.length === 0}
                  style={{ marginBottom: 8 }}
                >
                  🔍 识别页面类型
                </button>
                <button
                  className="btn btn-secondary"
                  onClick={handleStartProcessing}
                  disabled={progress.status === "processing" || files.length === 0}
                  style={{ width: "100%" }}
                >
                  {progress.status === "processing" ? "处理中..." : "直接处理（无分类）"}
                </button>
              </>
            ) : (
              <button
                className="btn btn-secondary"
                onClick={() => {
                  setWorkflowStage("idle");
                  setClassifications([]);
                  setNamingPreview([]);
                }}
                style={{ width: "100%" }}
              >
                ← 重新识别
              </button>
            )}
          </div>
        </aside>

        <main className="content-area">
          <div className="preview-header">
            <h3>处理结果</h3>
            <div className="preview-tabs">
              {workflowStage === "classified" && (
                <button
                  className={`preview-tab ${activeTab === "classify" ? "active" : ""}`}
                  onClick={() => setActiveTab("classify")}
                >
                  页面分类
                </button>
              )}
              <button
                className={`preview-tab ${activeTab === "preview" ? "active" : ""}`}
                onClick={() => setActiveTab("preview")}
              >
                发票列表
              </button>
              <button
                className={`preview-tab ${activeTab === "logs" ? "active" : ""}`}
                onClick={() => setActiveTab("logs")}
              >
                处理日志
              </button>
            </div>
          </div>

          <div className="preview-content">
            {activeTab === "classify" ? (
              <PageClassificationPanel
                classifications={classifications}
                namingPreview={namingPreview}
                onConfirm={handleConfirmBatchSplit}
                onClassificationsChange={setClassifications}
                isProcessing={workflowStage === "processing"}
              />
            ) : activeTab === "preview" ? (
              <PreviewPanel
                invoices={invoices}
                progress={progress}
                successCount={successCount}
                failedCount={failedCount}
                onCorrect={handleCorrectInvoice}
              />
            ) : (
              <LogPanel logs={logs} logRef={logRef} />
            )}
          </div>
        </main>
      </div>

      {correctingInvoice && (
        <div className="correction-modal">
          <div className="correction-dialog">
            <h3>人工校正发票信息</h3>
            <div className="form-group">
              <label>发票号码</label>
              <input
                type="text"
                value={correctingInvoice.invoiceNumber}
                onChange={(e) =>
                  setCorrectingInvoice({ ...correctingInvoice, invoiceNumber: e.target.value })
                }
              />
            </div>
            <div className="form-group">
              <label>购买方</label>
              <input
                type="text"
                value={correctingInvoice.buyer}
                onChange={(e) =>
                  setCorrectingInvoice({ ...correctingInvoice, buyer: e.target.value })
                }
              />
            </div>
            <div className="form-group">
              <label>销售方</label>
              <input
                type="text"
                value={correctingInvoice.seller}
                onChange={(e) =>
                  setCorrectingInvoice({ ...correctingInvoice, seller: e.target.value })
                }
              />
            </div>
            <div className="form-group">
              <label>金额</label>
              <input
                type="text"
                value={correctingInvoice.amount}
                onChange={(e) =>
                  setCorrectingInvoice({ ...correctingInvoice, amount: e.target.value })
                }
              />
            </div>
            <div className="form-group">
              <label>日期</label>
              <input
                type="text"
                value={correctingInvoice.date}
                onChange={(e) =>
                  setCorrectingInvoice({ ...correctingInvoice, date: e.target.value })
                }
              />
            </div>
            <div className="form-group">
              <label>票种</label>
              <input
                type="text"
                value={correctingInvoice.invoiceType}
                onChange={(e) =>
                  setCorrectingInvoice({ ...correctingInvoice, invoiceType: e.target.value })
                }
              />
            </div>
            <div className="form-group">
              <label>报销人</label>
              <input
                type="text"
                value={correctingInvoice.reimburser}
                onChange={(e) =>
                  setCorrectingInvoice({ ...correctingInvoice, reimburser: e.target.value })
                }
              />
            </div>
            <div className="form-group">
              <label>项目编号</label>
              <input
                type="text"
                value={correctingInvoice.projectCode}
                onChange={(e) =>
                  setCorrectingInvoice({ ...correctingInvoice, projectCode: e.target.value })
                }
              />
            </div>
            <div className="correction-actions">
              <button
                className="btn btn-secondary"
                onClick={() => setCorrectingInvoice(null)}
              >
                取消
              </button>
              <button
                className="btn btn-primary"
                onClick={() => handleSaveCorrection(correctingInvoice)}
              >
                保存
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
