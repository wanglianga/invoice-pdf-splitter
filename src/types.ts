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
