export interface InvoiceInfo {
  invoiceNumber: string;
  buyer: string;
  seller: string;
  amount: string;
  date: string;
  invoiceType: string;
  isReimbursement: boolean;
  pageNumber: number;
  originalFile: string;
  reimburser: string;
  projectCode: string;
  pageType: string;
  pageAction: string;
}

export interface ProcessedInvoice extends InvoiceInfo {
  outputFileName: string;
  outputPath: string;
  success: boolean;
  errorMessage?: string;
}

export interface ProcessingProgress {
  currentFile: string;
  currentPage: number;
  totalPages: number;
  processed: number;
  total: number;
  status: "idle" | "processing" | "completed" | "failed";
}

export interface AppSettings {
  namingTemplate: string;
  outputDirectory: string;
  duplicateHandling: "overwrite" | "skip" | "rename";
  manualCorrection: boolean;
  enableLog: boolean;
}

export interface LogEntry {
  timestamp: string;
  level: "info" | "warn" | "error" | "success";
  message: string;
}

export type PageType = "增值税发票" | "行程单" | "报销封面" | "附件" | "空白页" | "未知";

export type PageAction = "split" | "manual" | "merge" | "skip";

export interface PageClassification {
  pageNumber: number;
  originalFile: string;
  pageType: string;
  pageAction: string;
  confidence: number;
  info: InvoiceInfo;
}

export interface NamingPreview {
  template: string;
  fileName: string;
  warnings: NamingWarning[];
}

export interface NamingWarning {
  warningType: string;
  message: string;
  field?: string;
}

export const PAGE_TYPE_CONFIG: Record<string, { label: string; color: string; actionLabel: string; bgColor: string }> = {
  "增值税发票": { label: "增值税发票", color: "#1890ff", actionLabel: "可拆分", bgColor: "#e6f7ff" },
  "行程单": { label: "行程单", color: "#722ed1", actionLabel: "可拆分", bgColor: "#f9f0ff" },
  "报销封面": { label: "报销封面", color: "#fa8c16", actionLabel: "需人工判断", bgColor: "#fff7e6" },
  "附件": { label: "附件", color: "#13c2c2", actionLabel: "合并附件", bgColor: "#e6fffb" },
  "空白页": { label: "空白页", color: "#999", actionLabel: "无效页", bgColor: "#f5f5f5" },
  "未知": { label: "未知", color: "#ff4d4f", actionLabel: "需人工判断", bgColor: "#fff2f0" },
};

export const PAGE_ACTION_OPTIONS: { value: PageAction; label: string }[] = [
  { value: "split", label: "可拆分" },
  { value: "manual", label: "需人工判断" },
  { value: "merge", label: "合并附件" },
  { value: "skip", label: "跳过（无效）" },
];
