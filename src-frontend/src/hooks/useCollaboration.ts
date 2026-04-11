import { useEffect, useRef, useState, useCallback } from 'react';
import toast from 'react-hot-toast';
import type { TextOperation } from '@/types/collab';

interface CursorPosition {
  line: number;
  column: number;
}

interface Participant {
  user_id: string;
  user_name: string;
}

interface CollabMessage {
  type: 'join' | 'leave' | 'operation' | 'cursor' | 'ack' | 'sync' | 'error' | 'participants';
  session_id?: string;
  user_id?: string;
  user_name?: string;
  operation?: TextOperation;
  client_version?: number;
  position?: CursorPosition;
  version?: number;
  content?: string;
  message?: string;
  participants?: Participant[];
}

interface UseCollaborationOptions {
  storyId: string;
  chapterId: string;
  userId: string;
  userName: string;
  onRemoteOperation?: (op: TextOperation) => void;
  onUserJoined?: (user: Participant) => void;
  onUserLeft?: (user: Participant) => void;
}

export function useCollaboration({
  storyId,
  chapterId,
  userId,
  userName,
  onRemoteOperation,
  onUserJoined,
  onUserLeft,
}: UseCollaborationOptions) {
  const [isConnected, setIsConnected] = useState(false);
  const [version, setVersion] = useState(0);
  const [participants, setParticipants] = useState<Participant[]>([]);
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout>>();

  const connect = useCallback(() => {
    if (!storyId || !chapterId || !userId) {
      console.log('Cannot connect: missing storyId, chapterId or userId');
      return;
    }

    console.log(`Connecting to WebSocket for story ${storyId}, chapter ${chapterId}`);
    const ws = new WebSocket(`ws://localhost:8765`);

    ws.onopen = () => {
      console.log('WebSocket connected');
      setIsConnected(true);
      // Join session
      const joinMsg: CollabMessage = {
        type: 'join',
        session_id: `${storyId}-${chapterId}`,
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
            if (msg.operation && onRemoteOperation) {
              onRemoteOperation(msg.operation);
            }
            break;
          case 'participants':
            if (msg.participants) {
              setParticipants(msg.participants);
            }
            break;
          case 'ack':
            if (msg.version !== undefined) {
              setVersion(msg.version);
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
      console.log('WebSocket disconnected');
      setIsConnected(false);
      setParticipants([]);
    };

    ws.onerror = (error) => {
      console.error('WebSocket error:', error);
      toast.error('协同编辑连接失败，请检查网络');
    };

    wsRef.current = ws;
  }, [storyId, chapterId, userId, userName, onRemoteOperation]);

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
    setIsConnected(false);
    setParticipants([]);
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

  return {
    isConnected,
    version,
    participants,
    connect,
    disconnect,
    sendOperation,
    sendCursorPosition,
  };
}
