import type { AppSettings } from "../types";

interface SettingsPanelProps {
  settings: AppSettings;
  onSettingsChange: (settings: AppSettings) => void;
  onSelectOutputDir: () => void;
}

export default function SettingsPanel({
  settings,
  onSettingsChange,
  onSelectOutputDir,
}: SettingsPanelProps) {
  const templateVariables = [
    { key: "发票号码", value: "{发票号码}" },
    { key: "购买方", value: "{购买方}" },
    { key: "销售方", value: "{销售方}" },
    { key: "金额", value: "{金额}" },
    { key: "日期", value: "{日期}" },
    { key: "票种", value: "{票种}" },
    { key: "原页码", value: "{原页码}" },
  ];

  const insertVariable = (variable: string) => {
    onSettingsChange({
      ...settings,
      namingTemplate: settings.namingTemplate + variable,
    });
  };

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
