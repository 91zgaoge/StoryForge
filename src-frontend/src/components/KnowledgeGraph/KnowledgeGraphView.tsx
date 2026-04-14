import React, { useCallback, useEffect, useMemo, useState } from 'react';
import ReactFlow, {
  Background,
  Controls,
  MiniMap,
  useNodesState,
  useEdgesState,
  Node,
  Edge,
  MarkerType,
  Panel,
} from 'reactflow';
import 'reactflow/dist/style.css';
import type { Entity, Relation, EntityType } from '@/types/v3';
import { cn } from '@/utils/cn';

interface KnowledgeGraphViewProps {
  entities: Entity[];
  relations: Relation[];
  onNodeClick?: (entity: Entity) => void;
  className?: string;
}

const ENTITY_COLORS: Record<EntityType, string> = {
  Character: '#c96442',    // Terracotta
  Location: '#5b8c5a',     // Sage green
  Item: '#d4af37',         // Cinema gold
  Organization: '#6b5b95', // Purple
  Concept: '#4a90a4',      // Teal
  Event: '#c75b39',        // Rust
};

const ENTITY_LABELS: Record<EntityType, string> = {
  Character: '角色',
  Location: '地点',
  Item: '物品',
  Organization: '组织',
  Concept: '概念',
  Event: '事件',
};

function calculateLayout(entities: Entity[], relations: Relation[]) {
  const nodeMap = new Map<string, Node>();
  const typeCounts: Record<string, number> = {};
  const typeIndices: Record<string, number> = {};

  // Group by type
  entities.forEach((entity) => {
    typeCounts[entity.entity_type] = (typeCounts[entity.entity_type] || 0) + 1;
  });

  const centerX = 400;
  const centerY = 300;
  const radiusBase = 180;

  // Arrange in concentric circles by type
  const typeOrder: EntityType[] = ['Character', 'Location', 'Organization', 'Event', 'Concept', 'Item'];

  typeOrder.forEach((type, typeIndex) => {
    const count = typeCounts[type] || 0;
    if (count === 0) return;

    const radius = radiusBase + typeIndex * 120;
    const angleStep = (2 * Math.PI) / Math.max(count, 1);
    let currentAngle = typeIndex * 0.3; // Offset each ring

    entities
      .filter((e) => e.entity_type === type)
      .forEach((entity) => {
        const x = centerX + radius * Math.cos(currentAngle);
        const y = centerY + radius * Math.sin(currentAngle);
        currentAngle += angleStep;

        nodeMap.set(entity.id, {
          id: entity.id,
          position: { x, y },
          data: { entity },
          type: 'default',
          style: {
            background: ENTITY_COLORS[type],
            color: '#fff',
            border: '2px solid rgba(255,255,255,0.2)',
            borderRadius: '8px',
            padding: '8px 12px',
            fontSize: '13px',
            fontWeight: 500,
            minWidth: 80,
            textAlign: 'center',
            boxShadow: '0 2px 8px rgba(0,0,0,0.3)',
          },
        });
      });
  });

  const edges: Edge[] = relations.map((relation) => ({
    id: relation.id,
    source: relation.source_id,
    target: relation.target_id,
    label: relation.relation_type,
    type: 'smoothstep',
    animated: relation.strength > 0.7,
    style: {
      stroke: `rgba(212, 175, 55, ${0.3 + relation.strength * 0.7})`,
      strokeWidth: 1 + relation.strength * 3,
    },
    labelStyle: {
      fill: '#a0a0a0',
      fontSize: 11,
      fontWeight: 400,
    },
    labelBgStyle: {
      fill: '#1a1a1a',
      fillOpacity: 0.8,
    },
    labelBgPadding: [4, 4],
    labelShowBg: true,
    markerEnd: {
      type: MarkerType.ArrowClosed,
      color: `rgba(212, 175, 55, ${0.4 + relation.strength * 0.6})`,
    },
  }));

  return { nodes: Array.from(nodeMap.values()), edges };
}

export const KnowledgeGraphView: React.FC<KnowledgeGraphViewProps> = ({
  entities,
  relations,
  onNodeClick,
  className,
}) => {
  const [nodes, setNodes, onNodesChange] = useNodesState([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState([]);
  const [selectedEntity, setSelectedEntity] = useState<Entity | null>(null);
  const [fitView, setFitView] = useState(true);

  const layout = useMemo(
    () => calculateLayout(entities, relations),
    [entities, relations]
  );

  useEffect(() => {
    setNodes(layout.nodes);
    setEdges(layout.edges);
  }, [layout, setNodes, setEdges]);

  const handleNodeClick = useCallback(
    (_: React.MouseEvent, node: Node) => {
      const entity = entities.find((e) => e.id === node.id);
      if (entity) {
        setSelectedEntity(entity);
        onNodeClick?.(entity);
      }
    },
    [entities, onNodeClick]
  );

  const entityRelations = useMemo(() => {
    if (!selectedEntity) return [];
    return relations.filter(
      (r) => r.source_id === selectedEntity.id || r.target_id === selectedEntity.id
    );
  }, [selectedEntity, relations]);

  const getConnectedEntity = (relation: Relation) => {
    const otherId =
      relation.source_id === selectedEntity?.id
        ? relation.target_id
        : relation.source_id;
    return entities.find((e) => e.id === otherId);
  };

  return (
    <div className={cn('relative w-full h-full bg-cinema-950', className)}>
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onNodeClick={handleNodeClick}
        fitView={fitView}
        onInit={() => setFitView(false)}
        minZoom={0.2}
        maxZoom={2}
        proOptions={{ hideAttribution: true }}
      >
        <Background color="#333" gap={20} size={1} />
        <Controls className="bg-cinema-900 border-cinema-800" />
        <MiniMap
          nodeColor={(node) => {
            const type = (node.data?.entity as Entity)?.entity_type;
            return type ? ENTITY_COLORS[type] : '#666';
          }}
          className="bg-cinema-900 border-cinema-800"
          maskColor="rgba(0,0,0,0.5)"
        />
        <Panel position="top-left" className="bg-cinema-900/90 border border-cinema-800 rounded-xl p-3 m-2">
          <h3 className="text-sm font-semibold text-white mb-2">图例</h3>
          <div className="space-y-1.5">
            {(Object.keys(ENTITY_COLORS) as EntityType[]).map((type) => (
              <div key={type} className="flex items-center gap-2">
                <span
                  className="w-3 h-3 rounded-sm"
                  style={{ backgroundColor: ENTITY_COLORS[type] }}
                />
                <span className="text-xs text-gray-300">{ENTITY_LABELS[type]}</span>
              </div>
            ))}
          </div>
          <div className="mt-3 pt-2 border-t border-cinema-800 text-xs text-gray-500">
            <p>节点: {entities.length}</p>
            <p>关系: {relations.length}</p>
          </div>
        </Panel>
      </ReactFlow>

      {/* Entity Detail Panel */}
      {selectedEntity && (
        <div className="absolute right-4 top-4 bottom-4 w-72 bg-cinema-900/95 border border-cinema-800 rounded-xl p-4 overflow-y-auto shadow-2xl backdrop-blur-sm">
          <div className="flex items-start justify-between mb-3">
            <div>
              <span
                className="inline-block px-2 py-0.5 rounded text-[10px] font-medium text-white mb-1"
                style={{ backgroundColor: ENTITY_COLORS[selectedEntity.entity_type] }}
              >
                {ENTITY_LABELS[selectedEntity.entity_type]}
              </span>
              <h3 className="text-lg font-bold text-white">{selectedEntity.name}</h3>
            </div>
            <button
              onClick={() => setSelectedEntity(null)}
              className="text-gray-500 hover:text-white transition-colors"
            >
              ×
            </button>
          </div>

          {selectedEntity.attributes && Object.keys(selectedEntity.attributes).length > 0 && (
            <div className="mb-4">
              <h4 className="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-2">属性</h4>
              <div className="space-y-1.5">
                {Object.entries(selectedEntity.attributes).map(([key, value]) => (
                  <div key={key} className="text-sm">
                    <span className="text-cinema-gold">{key}:</span>{' '}
                    <span className="text-gray-300">
                      {typeof value === 'string' ? value : JSON.stringify(value)}
                    </span>
                  </div>
                ))}
              </div>
            </div>
          )}

          <div className="mb-4">
            <h4 className="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-2">关系</h4>
            {entityRelations.length === 0 ? (
              <p className="text-sm text-gray-500">暂无关系</p>
            ) : (
              <div className="space-y-2">
                {entityRelations.map((relation) => {
                  const other = getConnectedEntity(relation);
                  const isSource = relation.source_id === selectedEntity.id;
                  return (
                    <div
                      key={relation.id}
                      className="p-2 bg-cinema-800/50 rounded-lg text-sm"
                    >
                      <div className="flex items-center gap-1 text-gray-300">
                        <span className={isSource ? 'text-cinema-gold' : 'text-gray-300'}>
                          {selectedEntity.name}
                        </span>
                        <span className="text-gray-500">→</span>
                        <span className={!isSource ? 'text-cinema-gold' : 'text-gray-300'}>
                          {other?.name || '未知'}
                        </span>
                      </div>
                      <div className="flex items-center justify-between mt-1">
                        <span className="text-xs text-gray-400">{relation.relation_type}</span>
                        <div className="flex items-center gap-1">
                          <div
                            className="h-1 rounded-full bg-cinema-gold"
                            style={{ width: `${relation.strength * 24}px`, opacity: 0.6 + relation.strength * 0.4 }}
                          />
                          <span className="text-[10px] text-gray-500">
                            {Math.round(relation.strength * 100)}%
                          </span>
                        </div>
                      </div>
                    </div>
                  );
                })}
              </div>
            )}
          </div>

          <div className="text-xs text-gray-600 pt-3 border-t border-cinema-800">
            <p>首次出现: {new Date(selectedEntity.first_seen).toLocaleDateString()}</p>
          </div>
        </div>
      )}
    </div>
  );
};

export default KnowledgeGraphView;
