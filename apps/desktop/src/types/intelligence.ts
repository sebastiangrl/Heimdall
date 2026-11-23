export interface OllamaStatus {
  available: boolean;
  baseUrl: string;
  models: string[];
  message: string;
}

export interface AssistantReply {
  text: string;
  model: string;
  blockedReason?: string | null;
  queued?: boolean;
}

export interface AssistantQueueStatus {
  pending: boolean;
  questionPreview?: string | null;
  queuedAt?: string | null;
}