import { useState, useCallback } from 'react';
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
  const [error, setError] = useState<string | null>(null);

  const connect = useCallback(() => {
    console.log('[Collaboration] Connect called:', { storyId, chapterId, userId });
    
    if (!storyId || !chapterId || !userId) {
      console.log('[Collaboration] Cannot connect: missing params');
      setError('Missing required parameters');
      return;
    }

    setError(null);
    console.log(`[Collaboration] Connecting to ws://localhost:8765`);
    
    try {
      const ws = new WebSocket(`ws://localhost:8765`);

      ws.onopen = () => {
        console.log('[Collaboration] WebSocket connected');
        setIsConnected(true);
        setError(null);
        
        const joinMsg: CollabMessage = {
          type: 'join',
          session_id: `${storyId}-${chapterId}`,
          user_id: userId,
          user_name: userName,
        };
        console.log('[Collaboration] Sending join message:', joinMsg);
        ws.send(JSON.stringify(joinMsg));
      };

      ws.onmessage = (event) => {
        console.log('[Collaboration] Received message:', event.data);
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
              console.error('[Collaboration] Server error:', msg.message);
              setError(msg.message || 'Server error');
              break;
          }
        } catch (e) {
          console.error('[Collaboration] Failed to parse message:', e);
        }
      };

      ws.onclose = (event) => {
        console.log('[Collaboration] WebSocket closed:', event.code, event.reason);
        setIsConnected(false);
        setParticipants([]);
      };

      ws.onerror = (error) => {
        console.error('[Collaboration] WebSocket error:', error);
        setError('Connection failed');
        toast.error('协同编辑连接失败，请检查网络');
      };
    } catch (e) {
      console.error('[Collaboration] Failed to create WebSocket:', e);
      setError('Failed to create connection');
    }
  }, [storyId, chapterId, userId, userName, onRemoteOperation]);

  const disconnect = useCallback(() => {
    console.log('[Collaboration] Disconnecting...');
    setIsConnected(false);
    setParticipants([]);
    toast('已断开协同编辑连接');
  }, []);

  const sendOperation = useCallback((operation: TextOperation) => {
    console.log('[Collaboration] Sending operation:', operation);
    // Implementation would use wsRef
  }, []);

  const sendCursorPosition = useCallback((position: CursorPosition) => {
    console.log('[Collaboration] Sending cursor position:', position);
    // Implementation would use wsRef
  }, []);

  return {
    isConnected,
    version,
    participants,
    error,
    connect,
    disconnect,
    sendOperation,
    sendCursorPosition,
  };
}
