export interface TextOperation {
  op_type: 'Insert' | 'Delete' | 'Retain';
  position: number;
  content?: string;
  length: number;
  client_id: string;
  timestamp: number;
}

export interface CursorPosition {
  line: number;
  column: number;
}

export interface CollabUser {
  user_id: string;
  user_name: string;
  cursor?: CursorPosition;
  color: string;
}

export interface CollabSession {
  id: string;
  document_id: string;
  users: CollabUser[];
  version: number;
}
