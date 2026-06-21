import { useMemo } from "react";
import type { AppSettings, NamingPreview, NamingWarning, PageClassification } from "../types";

interface SettingsPanelProps {
  settings: AppSettings;
  onSettingsChange: (settings: AppSettings) => void;
  onSelectOutputDir: () => void;
  namingPreview: NamingPreview[];
  classifications: PageClassification[];
}

export default function SettingsPanel({
  settings,
  onSettingsChange,
  onSelectOutputDir,
  namingPreview,
  classifications,
}: SettingsPanelProps) {
  const templateVariables = [
    { key: "发票号码", value: "{发票号码}" },
    { key: "购买方", value: "{购买方}" },
    { key: "销售方", value: "{销售方}" },
    { key: "金额", value: "{金额}" },
    { key: "日期", value: "{日期}" },
    { key: "票种", value: "{票种}" },
    { key: "原页码", value: "{原页码}" },
    { key: "报销人", value: "{报销人}" },
    { key: "项目编号", value: "{项目编号}" },
  ];

  const insertVariable = (variable: string) => {
    onSettingsChange({
      ...settings,
      namingTemplate: settings.namingTemplate + variable,
    });
  };

  const previewFileName = useMemo(() => {
    if (namingPreview.length > 0) {
      return namingPreview[0].fileName;
    }
    if (classifications.length > 0) {
      return "(添加文件后预览)";
    }
    return "(请添加文件后预览)";
  }, [namingPreview, classifications]);

  const allWarnings = useMemo(() => {
    const warnings: NamingWarning[] = [];
    const seen = new Set<string>();
    for (const p of namingPreview) {
      for (const w of p.warnings) {
        const key = `${w.warningType}:${w.message}`;
        if (!seen.has(key)) {
          seen.add(key);
          warnings.push(w);
        }
      }
    }
    return warnings;
  }, [namingPreview]);

  const duplicateWarnings = useMemo(() => {
    return allWarnings.filter((w) => w.warningType === "duplicate_name");
  }, [allWarnings]);

  const otherWarnings = useMemo(() => {
    return allWarnings.filter((w) => w.warningType !== "duplicate_name");
  }, [allWarnings]);

  return (
    <div className="sidebar-section">
      <h3>⚙️ 设置</h3>

      <div className="form-group">
        <label>命名模板</label>
        <input
          type="text"
          value={settings.namingTemplate}
          onChange={(e) =>
            onSettingsChange({ ...settings, namingTemplate: e.target.value })
          }
        />
        <div className="hint">可用变量：</div>
        <div className="template-variables">
          {templateVariables.map((v) => (
            <span
              key={v.key}
              className="template-var"
              onClick={() => insertVariable(v.value)}
              title={`点击插入 ${v.key}`}
            >
              {v.value}
            </span>
          ))}
        </div>
      </div>

      <div className="form-group">
        <label>文件名预览</label>
        <div className="naming-preview-box">
          <div className="naming-preview-filename">{previewFileName}</div>
          {namingPreview.length > 1 && (
            <div className="naming-preview-count">
              共 {namingPreview.length} 个文件
            </div>
          )}
        </div>
        {otherWarnings.length > 0 && (
          <div className="naming-warnings">
            {otherWarnings.map((w, idx) => (
              <div key={idx} className={`naming-warning-item warning-${w.warningType}`}>
                <span className="warning-icon">
                  {w.warningType === "illegal_char" ? "⚠️" :
                   w.warningType === "path_too_long" ? "📏" :
                   w.warningType === "missing_field" ? "❓" :
                   w.warningType === "placeholder_in_name" ? "📋" : "⚠️"}
                </span>
                <span>{w.message}</span>
              </div>
            ))}
          </div>
        )}
        {duplicateWarnings.length > 0 && (
          <div className="naming-warnings">
            {duplicateWarnings.slice(0, 3).map((w, idx) => (
              <div key={idx} className="naming-warning-item warning-duplicate_name">
                <span className="warning-icon">🔄</span>
                <span>{w.message}</span>
              </div>
            ))}
            {duplicateWarnings.length > 3 && (
              <div className="naming-warning-more">
                还有 {duplicateWarnings.length - 3} 个重复名称警告...
              </div>
            )}
          </div>
        )}
      </div>

      <div className="form-group">
        <label>输出目录</label>
        <div style={{ display: "flex", gap: 8 }}>
          <input
            type="text"
            value={settings.outputDirectory}
            readOnly
            placeholder="请选择输出目录"
            style={{ flex: 1 }}
          />
          <button
            className="btn btn-secondary btn-small"
            onClick={onSelectOutputDir}
          >
            选择
          </button>
        </div>
      </div>

      <div className="form-group">
        <label>重复文件处理</label>
        <select
          value={settings.duplicateHandling}
          onChange={(e) =>
            onSettingsChange({
              ...settings,
              duplicateHandling: e.target.value as AppSettings["duplicateHandling"],
            })
          }
        >
          <option value="rename">自动重命名</option>
          <option value="overwrite">覆盖</option>
          <option value="skip">跳过</option>
        </select>
      </div>

      <div className="form-group">
        <label className="checkbox-group">
          <input
            type="checkbox"
            checked={settings.manualCorrection}
            onChange={(e) =>
              onSettingsChange({ ...settings, manualCorrection: e.target.checked })
            }
          />
          启用人工校正
        </label>
      </div>

      <div className="form-group">
        <label className="checkbox-group">
          <input
            type="checkbox"
            checked={settings.enableLog}
            onChange={(e) =>
              onSettingsChange({ ...settings, enableLog: e.target.checked })
            }
          />
          启用处理日志
        </label>
      </div>
    </div>
  );
}
