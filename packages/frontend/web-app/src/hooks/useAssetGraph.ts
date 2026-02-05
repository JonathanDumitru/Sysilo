import { useMemo } from 'react';
import type { Node, Edge } from '@xyflow/react';
import type { Asset } from '../services/assets.js';
import type { AssetNodeData } from '../components/graph/index.js';

interface AssetGraphData {
  nodes: Node<AssetNodeData>[];
  edges: Edge[];
}

/**
 * Transforms assets into React Flow nodes and edges.
 * Edges connect assets that share vendors, teams, or tags.
 */
export function useAssetGraph(assets: Asset[]): AssetGraphData {
  return useMemo(() => {
    if (!assets.length) {
      return { nodes: [], edges: [] };
    }

    // Create nodes - arrange in a grid initially (React Flow will auto-layout)
    const COLS = 4;
    const X_SPACING = 250;
    const Y_SPACING = 150;

    const nodes: Node<AssetNodeData>[] = assets.map((asset, index) => ({
      id: asset.id,
      type: 'asset',
      position: {
        x: (index % COLS) * X_SPACING + 50,
        y: Math.floor(index / COLS) * Y_SPACING + 50,
      },
      data: {
        id: asset.id,
        name: asset.name,
        asset_type: asset.asset_type,
        status: asset.status,
        vendor: asset.vendor,
        tags: asset.tags,
      },
    }));

    // Create edges for related assets
    const edges: Edge[] = [];
    const edgeSet = new Set<string>(); // Prevent duplicates

    for (let i = 0; i < assets.length; i++) {
      for (let j = i + 1; j < assets.length; j++) {
        const a = assets[i];
        const b = assets[j];
        const edgeId = `${a.id}-${b.id}`;

        if (edgeSet.has(edgeId)) continue;

        // Connect by shared vendor
        if (a.vendor && b.vendor && a.vendor === b.vendor) {
          edges.push({
            id: `vendor-${edgeId}`,
            source: a.id,
            target: b.id,
            type: 'default',
            style: { stroke: '#94a3b8', strokeWidth: 1.5 },
            label: a.vendor,
            labelStyle: { fontSize: 10, fill: '#64748b' },
            labelBgStyle: { fill: '#f8fafc', fillOpacity: 0.9 },
            labelBgPadding: [4, 2] as [number, number],
            labelBgBorderRadius: 4,
          });
          edgeSet.add(edgeId);
          continue;
        }

        // Connect by shared team
        if (a.team && b.team && a.team === b.team) {
          edges.push({
            id: `team-${edgeId}`,
            source: a.id,
            target: b.id,
            type: 'default',
            style: { stroke: '#60a5fa', strokeWidth: 1.5 },
          });
          edgeSet.add(edgeId);
          continue;
        }

        // Connect by shared tags (at least one common tag)
        const sharedTags = a.tags.filter((tag) => b.tags.includes(tag));
        if (sharedTags.length > 0) {
          edges.push({
            id: `tags-${edgeId}`,
            source: a.id,
            target: b.id,
            type: 'default',
            style: { stroke: '#a78bfa', strokeWidth: 1, strokeDasharray: '5,5' },
          });
          edgeSet.add(edgeId);
        }
      }
    }

    return { nodes, edges };
  }, [assets]);
}
