import type { ProcessedInvoice, ProcessingProgress } from "../types";

interface PreviewPanelProps {
  invoices: ProcessedInvoice[];
  progress: ProcessingProgress;
  successCount: number;
  failedCount: number;
  onCorrect: (invoice: ProcessedInvoice) => void;
}

export default function PreviewPanel({
  invoices,
  progress,
  successCount,
  failedCount,
  onCorrect,
}: PreviewPanelProps) {
  if (progress.status === "idle" && invoices.length === 0) {
    return (
      <div className="empty-state">
        <div className="empty-state-icon">📊</div>
        <div className="empty-state-text">添加 PDF 文件后点击开始处理</div>
      </div>
    );
  }

  return (
    <div>
      {(progress.status === "processing" || progress.status === "completed") && (
        <div className="progress-section">
          <div className="progress-bar-container">
            <div className="progress-bar">
              <div
                className="progress-bar-fill"
                style={{
                  width: `${progress.total > 0 ? (progress.processed / progress.total) * 100 : 0}%`,
                }}
              />
            </div>
            <div className="progress-info">
              <span>
                当前文件: {progress.currentFile || "-"} (第 {progress.currentPage}/
                {progress.totalPages} 页)
              </span>
              <span>
                {progress.processed} / {progress.total}
              </span>
            </div>
          </div>
        </div>
      )}

      {invoices.length > 0 && (
        <div className="stats-row">
          <div className="stat-card">
            <div className="stat-value">{invoices.length}</div>
            <div className="stat-label">总计</div>
          </div>
          <div className="stat-card success">
            <div className="stat-value">{successCount}</div>
            <div className="stat-label">成功</div>
          </div>
          <div className="stat-card failed">
            <div className="stat-value">{failedCount}</div>
            <div className="stat-label">失败</div>
          </div>
        </div>
      )}

      <div className="invoice-list">
        {invoices.map((invoice, index) => (
          <div
            key={`${invoice.originalFile}-${invoice.pageNumber}-${index}`}
            className={`invoice-card ${invoice.success ? "success" : "failed"}`}
          >
            <div className="invoice-card-header">
              <span className="invoice-number">{invoice.invoiceNumber || "未识别"}</span>
              <span className={`status-badge ${invoice.success ? "status-success" : "status-failed"}`}>
                {invoice.success ? "成功" : "失败"}
              </span>
            </div>

            <div className="invoice-info-row">
              <span className="invoice-info-label">票种:</span>
              <span className="invoice-info-value">{invoice.invoiceType || "-"}</span>
            </div>

            <div className="invoice-info-row">
              <span className="invoice-info-label">销售方:</span>
              <span className="invoice-info-value" title={invoice.seller}>
                {invoice.seller || "-"}
              </span>
            </div>

            <div className="invoice-info-row">
              <span className="invoice-info-label">购买方:</span>
              <span className="invoice-info-value" title={invoice.buyer}>
                {invoice.buyer || "-"}
              </span>
            </div>

            <div className="invoice-info-row">
              <span className="invoice-info-label">日期:</span>
              <span className="invoice-info-value">{invoice.date || "-"}</span>
            </div>

            <div className="invoice-info-row">
              <span className="invoice-info-label">原页码:</span>
              <span className="invoice-info-value">第 {invoice.pageNumber} 页</span>
            </div>

            <div className="invoice-amount">¥ {invoice.amount || "0.00"}</div>

            {invoice.success ? (
              <div className="invoice-output">
                <div>输出文件名:</div>
                <div className="invoice-output-name">{invoice.outputFileName}</div>
              </div>
            ) : (
              <div className="invoice-output" style={{ color: "#ff4d4f" }}>
                <div>失败原因:</div>
                <div style={{ marginTop: 4 }}>{invoice.errorMessage}</div>
              </div>
            )}

            <div style={{ marginTop: 10, textAlign: "right" }}>
              <button
                className="btn btn-secondary btn-small"
                onClick={() => onCorrect(invoice)}
              >
                人工校正
              </button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
