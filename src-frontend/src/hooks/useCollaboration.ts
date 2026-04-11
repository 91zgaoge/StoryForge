import { useEffect, useRef, useState, useCallback } from 'react';
import type { TextOperation } from '@/types/collab';

interface CursorPosition {
  line: number;
  column: number;
}

interface CollabMessage {
  type: 'join' | 'leave' | 'operation' | 'cursor' | 'ack' | 'sync' | 'error';
  session_id?: string;
  user_id?: string;
  user_name?: string;
  operation?: TextOperation;
  client_version?: number;
  position?: CursorPosition;
  version?: number;
  content?: string;
  message?: string;
}

interface UseCollaborationOptions {
  sessionId: string;
  documentId: string;
  userId: string;
  userName: string;
  onOperation?: (op: TextOperation) => void;
  onCursorUpdate?: (userId: string, position: CursorPosition) => void;
  onSync?: (content: string, version: number) => void;
}

export function useCollaboration({
  sessionId,
  documentId,
  userId,
  userName,
  onOperation,
  onCursorUpdate,
  onSync,
}: UseCollaborationOptions) {
  const [isConnected, setIsConnected] = useState(false);
  const [version, setVersion] = useState(0);
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout>();

  const connect = useCallback(() => {
    const ws = new WebSocket(`ws://localhost:8765`);
    
    ws.onopen = () => {
      setIsConnected(true);
      // Join session
      const joinMsg: CollabMessage = {
        type: 'join',
        session_id: sessionId,
        user_id: userId,
        user_name: userName,
      };
      ws.send(JSON.stringify(joinMsg));
    };

    ws.onmessage = (event) => {
      try {
        const msg: CollabMessage = JSON.parse(event.data);
        
        switch (msg.type) {
          case 'operation':
            if (msg.operation && onOperation) {
              onOperation(msg.operation);
            }
            break;
          case 'cursor':
            if (msg.user_id && msg.position && onCursorUpdate) {
              onCursorUpdate(msg.user_id, msg.position);
            }
            break;
          case 'ack':
            if (msg.version !== undefined) {
              setVersion(msg.version);
            }
            break;
          case 'sync':
            if (msg.content !== undefined && msg.version !== undefined && onSync) {
              onSync(msg.content, msg.version);
            }
            break;
          case 'error':
            console.error('Collab error:', msg.message);
            break;
        }
      } catch (e) {
        console.error('Failed to parse message:', e);
      }
    };

    ws.onclose = () => {
      setIsConnected(false);
      // Reconnect after 3 seconds
      reconnectTimeoutRef.current = setTimeout(() => {
        connect();
      }, 3000);
    };

    ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };

    wsRef.current = ws;
  }, [sessionId, documentId, userId, userName, onOperation, onCursorUpdate, onSync]);

  const disconnect = useCallback(() => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
    }
    if (wsRef.current) {
      const leaveMsg: CollabMessage = {
        type: 'leave',
        user_id: userId,
      };
      wsRef.current.send(JSON.stringify(leaveMsg));
      wsRef.current.close();
    }
  }, [userId]);

  const sendOperation = useCallback((operation: TextOperation) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      const msg: CollabMessage = {
        type: 'operation',
        operation,
        client_version: version,
      };
      wsRef.current.send(JSON.stringify(msg));
    }
  }, [version]);

  const sendCursorPosition = useCallback((position: CursorPosition) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      const msg: CollabMessage = {
        type: 'cursor',
        user_id: userId,
        position,
      };
      wsRef.current.send(JSON.stringify(msg));
    }
  }, [userId]);

  useEffect(() => {
    connect();
    return () => disconnect();
  }, [connect, disconnect]);

  return {
    isConnected,
    version,
    sendOperation,
    sendCursorPosition,
  };
}
