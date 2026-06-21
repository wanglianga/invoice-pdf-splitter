import { useState, useMemo } from "react";
import type { PageClassification, NamingPreview, PageAction } from "../types";
import { PAGE_TYPE_CONFIG, PAGE_ACTION_OPTIONS } from "../types";

interface PageClassificationPanelProps {
  classifications: PageClassification[];
  namingPreview: NamingPreview[];
  onConfirm: (confirmedPages: PageClassification[]) => void;
  onClassificationsChange: (classifications: PageClassification[]) => void;
  isProcessing: boolean;
}

export default function PageClassificationPanel({
  classifications,
  namingPreview,
  onConfirm,
  onClassificationsChange,
  isProcessing,
}: PageClassificationPanelProps) {
  const [filter, setFilter] = useState<string>("all");
  const [editingPage, setEditingPage] = useState<number | null>(null);

  const stats = useMemo(() => {
    const result: Record<string, number> = {};
    for (const c of classifications) {
      result[c.pageType] = (result[c.pageType] || 0) + 1;
    }
    return result;
  }, [classifications]);

  const actionStats = useMemo(() => {
    const result: Record<string, number> = {};
    for (const c of classifications) {
      result[c.pageAction] = (result[c.pageAction] || 0) + 1;
    }
    return result;
  }, [classifications]);

  const filteredClassifications = useMemo(() => {
    if (filter === "all") return classifications;
    if (filter === "actionable") return classifications.filter((c) => c.pageAction !== "skip");
    return classifications.filter((c) => c.pageType === filter);
  }, [classifications, filter]);

  const handleActionChange = (pageNumber: number, newAction: PageAction) => {
    const updated = classifications.map((c) => {
      if (c.pageNumber === pageNumber) {
        let newType = c.pageType;
        if (newAction === "skip") newType = "空白页";
        else if (newAction === "merge") newType = "附件";
        else if (newAction === "manual") {
          if (c.pageType !== "增值税发票" && c.pageType !== "行程单" && c.pageType !== "报销封面") {
            newType = "未知";
          }
        } else if (newAction === "split") {
          if (c.pageType === "空白页" || c.pageType === "未知" || c.pageType === "附件") {
            newType = "增值税发票";
          }
        }
        return { ...c, pageAction: newAction, pageType: newType };
      }
      return c;
    });
    onClassificationsChange(updated);
  };

  const handleInfoEdit = (pageNumber: number, field: string, value: string) => {
    const updated = classifications.map((c) => {
      if (c.pageNumber === pageNumber) {
        const infoKeyMap: Record<string, string> = {
          invoiceNumber: "invoiceNumber",
          seller: "seller",
          buyer: "buyer",
          amount: "amount",
          date: "date",
          reimburser: "reimburser",
          projectCode: "projectCode",
        };
        const key = infoKeyMap[field];
        if (key) {
          return { ...c, info: { ...c.info, [key]: value } };
        }
      }
      return c;
    });
    onClassificationsChange(updated);
  };

  const getPreviewForPage = (pageNumber: number): NamingPreview | undefined => {
    return namingPreview.find((p) => {
      const pageIdx = pageNumber - 1;
      return pageIdx < namingPreview.length;
    });
  };

  const totalWarnings = useMemo(() => {
    return namingPreview.reduce((sum, p) => sum + p.warnings.length, 0);
  }, [namingPreview]);

  return (
    <div className="classification-panel">
      <div className="classification-stats">
        {Object.entries(stats).map(([type, count]) => {
          const config = PAGE_TYPE_CONFIG[type] || PAGE_TYPE_CONFIG["未知"];
          return (
            <div
              key={type}
              className="stat-chip"
              style={{ background: config.bgColor, color: config.color, cursor: "pointer", border: filter === type ? `2px solid ${config.color}` : "2px solid transparent" }}
              onClick={() => setFilter(filter === type ? "all" : type)}
            >
              <span className="stat-chip-count">{count}</span>
              <span className="stat-chip-label">{config.label}</span>
              <span className="stat-chip-action">{config.actionLabel}</span>
            </div>
          );
        })}
        <div
          className="stat-chip"
          style={{ background: "#e8f4fd", color: "#1890ff", cursor: "pointer", border: filter === "actionable" ? "2px solid #1890ff" : "2px solid transparent" }}
          onClick={() => setFilter(filter === "actionable" ? "all" : "actionable")}
        >
          <span className="stat-chip-count">{actionStats["split"] || 0}</span>
          <span className="stat-chip-label">可处理</span>
        </div>
      </div>

      <div className="classification-summary">
        <span>共 {classifications.length} 页</span>
        <span>可拆分: {actionStats["split"] || 0}</span>
        <span>需人工判断: {actionStats["manual"] || 0}</span>
        <span>合并附件: {actionStats["merge"] || 0}</span>
        <span>无效跳过: {actionStats["skip"] || 0}</span>
        {totalWarnings > 0 && (
          <span className="warning-count">命名警告: {totalWarnings}</span>
        )}
      </div>

      <div className="classification-list">
        {filteredClassifications.map((page) => {
          const config = PAGE_TYPE_CONFIG[page.pageType] || PAGE_TYPE_CONFIG["未知"];
          const preview = getPreviewForPage(page.pageNumber);
          const isEditing = editingPage === page.pageNumber;

          return (
            <div
              key={`${page.originalFile}-${page.pageNumber}`}
              className="classification-card"
              style={{ borderLeftColor: config.color }}
            >
              <div className="classification-card-header">
                <div className="classification-card-left">
                  <span className="page-number">第 {page.pageNumber} 页</span>
                  <span
                    className="page-type-badge"
                    style={{ background: config.bgColor, color: config.color }}
                  >
                    {config.label}
                  </span>
                  <span className="confidence-badge">
                    {Math.round(page.confidence * 100)}%
                  </span>
                </div>
                <div className="classification-card-right">
                  <select
                    className="action-select"
                    value={page.pageAction}
                    onChange={(e) => handleActionChange(page.pageNumber, e.target.value as PageAction)}
                  >
                    {PAGE_ACTION_OPTIONS.map((opt) => (
                      <option key={opt.value} value={opt.value}>
                        {opt.label}
                      </option>
                    ))}
                  </select>
                  <button
                    className="btn btn-secondary btn-small"
                    onClick={() => setEditingPage(isEditing ? null : page.pageNumber)}
                  >
                    {isEditing ? "收起" : "编辑"}
                  </button>
                </div>
              </div>

              <div className="classification-card-info">
                <span className="info-item" title="发票号码">
                  发票号: {page.info.invoiceNumber}
                </span>
                <span className="info-item" title="销售方">
                  销售方: {page.info.seller}
                </span>
                <span className="info-item" title="金额">
                  ¥ {page.info.amount}
                </span>
                <span className="info-item" title="日期">
                  {page.info.date}
                </span>
                {page.info.reimburser !== "未识别" && (
                  <span className="info-item" title="报销人">
                    报销人: {page.info.reimburser}
                  </span>
                )}
                {page.info.projectCode !== "未识别" && (
                  <span className="info-item" title="项目编号">
                    项目编号: {page.info.projectCode}
                  </span>
                )}
              </div>

              {preview && preview.warnings.length > 0 && (
                <div className="classification-warnings">
                  {preview.warnings.map((w, idx) => (
                    <div key={idx} className={`naming-warning-item warning-${w.warningType}`}>
                      <span className="warning-icon">
                        {w.warningType === "illegal_char" ? "⚠️" :
                         w.warningType === "path_too_long" ? "📏" :
                         w.warningType === "missing_field" ? "❓" :
                         w.warningType === "duplicate_name" ? "🔄" :
                         w.warningType === "placeholder_in_name" ? "📋" : "⚠️"}
                      </span>
                      <span>{w.message}</span>
                    </div>
                  ))}
                </div>
              )}

              {isEditing && (
                <div className="classification-edit-form">
                  <div className="edit-form-row">
                    <label>发票号码</label>
                    <input
                      type="text"
                      value={page.info.invoiceNumber}
                      onChange={(e) => handleInfoEdit(page.pageNumber, "invoiceNumber", e.target.value)}
                    />
                  </div>
                  <div className="edit-form-row">
                    <label>销售方</label>
                    <input
                      type="text"
                      value={page.info.seller}
                      onChange={(e) => handleInfoEdit(page.pageNumber, "seller", e.target.value)}
                    />
                  </div>
                  <div className="edit-form-row">
                    <label>购买方</label>
                    <input
                      type="text"
                      value={page.info.buyer}
                      onChange={(e) => handleInfoEdit(page.pageNumber, "buyer", e.target.value)}
                    />
                  </div>
                  <div className="edit-form-row">
                    <label>金额</label>
                    <input
                      type="text"
                      value={page.info.amount}
                      onChange={(e) => handleInfoEdit(page.pageNumber, "amount", e.target.value)}
                    />
                  </div>
                  <div className="edit-form-row">
                    <label>日期</label>
                    <input
                      type="text"
                      value={page.info.date}
                      onChange={(e) => handleInfoEdit(page.pageNumber, "date", e.target.value)}
                    />
                  </div>
                  <div className="edit-form-row">
                    <label>报销人</label>
                    <input
                      type="text"
                      value={page.info.reimburser}
                      onChange={(e) => handleInfoEdit(page.pageNumber, "reimburser", e.target.value)}
                    />
                  </div>
                  <div className="edit-form-row">
                    <label>项目编号</label>
                    <input
                      type="text"
                      value={page.info.projectCode}
                      onChange={(e) => handleInfoEdit(page.pageNumber, "projectCode", e.target.value)}
                    />
                  </div>
                </div>
              )}
            </div>
          );
        })}
      </div>

      <div className="classification-actions">
        <button
          className="btn btn-primary"
          onClick={() => onConfirm(classifications)}
          disabled={isProcessing || classifications.length === 0}
        >
          {isProcessing ? "处理中..." : `确认并批量输出 (${actionStats["split"] || 0} 页可拆分)`}
        </button>
      </div>
    </div>
  );
}
