// Graph visualization using D3.js

let svg = null;
let graphGroup = null;
let nodesGroup = null;
let edgesGroup = null;
let zoomBehavior = null;
let simulation = null;
let currentData = null;
let currentView = null;
let currentTransform = d3.zoomIdentity;
let layoutMode = 'hierarchy';
let searchQuery = '';
let currentLevel = null;
let hierarchyCache = null;
let suppressZoomUpdates = false;

const MAX_VISIBLE_NODES = 500;
const NODE_WIDTH = 140;
const NODE_HEIGHT = 44;

const LEVELS = [
    { key: 'workspace', name: 'Workspace', min: 0, max: 0.35 },
    { key: 'module', name: 'Modules', min: 0.35, max: 0.7 },
    { key: 'directory', name: 'Directories', min: 0.7, max: 1.1 },
    { key: 'file', name: 'Files', min: 1.1, max: 1.6 },
    { key: 'symbol', name: 'Symbols', min: 1.6, max: Infinity }
];

const SYMBOL_KINDS = new Set([
    'class',
    'struct',
    'enum',
    'interface',
    'function',
    'method',
    'constant',
    'typealias',
    'configblock',
    'configkey',
    'envvariable',
    'route',
    'migration',
    'cijob',
    'dockerservice',
    'unknown'
]);

const STRUCTURAL_KINDS = new Set([
    'directory',
    'file',
    'workspaceroot',
    'package',
    'module'
]);

function init() {
    svg = d3.select('#graph');
    graphGroup = svg.append('g').attr('id', 'graph-group');
    edgesGroup = graphGroup.append('g').attr('class', 'edges');
    nodesGroup = graphGroup.append('g').attr('class', 'nodes');

    addDefs();
    setupZoom();
    setupControls();
    connectWebSocket();
}

function addDefs() {
    const defs = svg.append('defs');
    createArrow(defs, 'arrow-contains', '#3d4b66');
    createArrow(defs, 'arrow-syntactic', '#79e0a5');
    createArrow(defs, 'arrow-semantic', '#5bd6ff');
    createArrow(defs, 'arrow-other', '#caa4ff');
}

function createArrow(defs, id, color) {
    defs.append('marker')
        .attr('id', id)
        .attr('viewBox', '0 -5 10 10')
        .attr('refX', 10)
        .attr('refY', 0)
        .attr('markerWidth', 6)
        .attr('markerHeight', 6)
        .attr('orient', 'auto')
        .append('path')
        .attr('d', 'M0,-5L10,0L0,5')
        .attr('fill', color);
}

function setupZoom() {
    zoomBehavior = d3.zoom()
        .scaleExtent([0.15, 4])
        .on('zoom', (event) => {
            currentTransform = event.transform;
            graphGroup.attr('transform', currentTransform);
            if (!suppressZoomUpdates) {
                handleZoomLevelChange(event.transform.k);
            }
        });

    svg.call(zoomBehavior);
}

function setupControls() {
    const searchInput = document.getElementById('search-input');
    const searchButton = document.getElementById('search-button');
    const layoutToggle = document.getElementById('layout-toggle');
    const fitButton = document.getElementById('fit-button');
    const resetButton = document.getElementById('reset-button');

    if (searchInput) {
        searchInput.addEventListener('keydown', (event) => {
            if (event.key === 'Enter') {
                searchNodes(event.target.value);
            }
        });
    }

    if (searchButton && searchInput) {
        searchButton.addEventListener('click', () => searchNodes(searchInput.value));
    }

    if (layoutToggle) {
        layoutToggle.addEventListener('click', () => {
            layoutMode = layoutMode === 'hierarchy' ? 'force' : 'hierarchy';
            layoutToggle.textContent = `Layout: ${layoutMode === 'hierarchy' ? 'Orbit' : 'Force'}`;
            if (currentView) {
                layoutAndRender(currentView, { refit: true });
            }
        });
    }

    if (fitButton) {
        fitButton.addEventListener('click', () => fitToView());
    }

    if (resetButton) {
        resetButton.addEventListener('click', () => resetView());
    }

    const filterIds = [
        'filter-directories',
        'filter-files',
        'filter-symbols',
        'filter-edges-contains',
        'filter-edges-syntactic',
        'filter-edges-semantic',
        'filter-edges-other'
    ];

    filterIds.forEach((id) => {
        const checkbox = document.getElementById(id);
        if (checkbox) {
            checkbox.addEventListener('change', () => applyFilters());
        }
    });

    window.addEventListener('resize', () => {
        if (currentView) {
            layoutAndRender(currentView, { refit: true });
        }
    });
}

function connectWebSocket() {
    if (window.WebSocketProtocol) {
        window.WebSocketProtocol.connect();
    } else {
        const wsUrl = `ws://${window.location.hostname}:7890/ws`;
        const ws = new WebSocket(wsUrl);

        ws.onopen = () => {
            updateStatus('Connected');
        };

        ws.onmessage = (event) => {
            const data = JSON.parse(event.data);
            if (data.type === 'graph_diff') {
                applyDiff(data.diff);
            } else if (data.type === 'full_graph') {
                renderGraph(data.graph);
            }
        };

        ws.onerror = () => updateStatus('Connection error');
        ws.onclose = () => updateStatus('Disconnected');
    }
}

function renderGraph(graphData) {
    if (!graphData || !graphData.nodes) {
        return;
    }

    const normalized = normalizeGraph(graphData);
    currentData = normalized;
    hierarchyCache = buildHierarchyCache(normalized);
    updateView({ refit: true, reason: 'data' });
}

function updateView({ refit } = {}) {
    if (!currentData) {
        return;
    }

    const desiredLevel = levelForScale(currentTransform.k || 1);
    const result = computeViewGraph(currentData, hierarchyCache, desiredLevel);

    currentLevel = result.levelIndex;
    currentView = result.viewData;
    currentView.levelIndex = result.levelIndex;
    currentView.isCapped = result.capped;
    currentView.desiredLevel = desiredLevel;

    layoutAndRender(currentView, { refit });
}

function handleZoomLevelChange(scale) {
    const desiredLevel = levelForScale(scale);
    if (currentView && currentView.desiredLevel === desiredLevel && currentLevel === currentView.levelIndex) {
        return;
    }
    updateView({ refit: true });
}

function levelForScale(scale) {
    const match = LEVELS.find((level) => scale >= level.min && scale < level.max);
    return match ? LEVELS.indexOf(match) : LEVELS.length - 1;
}

function computeViewGraph(data, hierarchy, desiredLevel) {
    let levelIndex = Math.max(0, Math.min(desiredLevel, LEVELS.length - 1));
    let viewData = buildViewGraph(data, hierarchy, levelIndex);
    let capped = false;

    while (viewData.nodes.length > MAX_VISIBLE_NODES && levelIndex > 0) {
        capped = true;
        levelIndex -= 1;
        viewData = buildViewGraph(data, hierarchy, levelIndex);
    }

    if (viewData.nodes.length > MAX_VISIBLE_NODES) {
        capped = true;
        viewData = buildViewGraph(data, hierarchy, Math.max(levelIndex, 0), MAX_VISIBLE_NODES);
    }

    return { viewData, levelIndex, capped };
}

function buildViewGraph(data, hierarchy, levelIndex, capLimit) {
    const levelKey = LEVELS[levelIndex].key;
    const seeds = selectSeeds(data, hierarchy, levelKey);
    let visible = new Set();

    if (seeds.length === 0) {
        hierarchy.roots.forEach((rootId) => visible.add(rootId));
    } else {
        seeds.forEach((id) => includeWithAncestors(id, visible, hierarchy.parentById));
    }

    if (capLimit && visible.size > capLimit) {
        visible = capVisibleSet(visible, data, hierarchy, capLimit);
    }

    const nodes = [];
    visible.forEach((id) => {
        const node = data.nodeById.get(id);
        if (!node) {
            return;
        }
        const depth = hierarchy.depthById.get(id) || 0;
        const displayKind = displayKindForLevel(node, levelKey, depth);
        nodes.push({ ...node, displayKind });
    });

    const nodeById = new Map(nodes.map((node) => [node.id, node]));
    const containsEdges = buildContainsEdges(nodes, visible, hierarchy.parentById, nodeById);
    const aggregatedEdges = buildAggregatedEdges(data, visible, hierarchy.parentById, nodeById);

    const edges = containsEdges.concat(aggregatedEdges);

    edges.forEach((edge) => {
        edge.sourceNode = nodeById.get(edge.source);
        edge.targetNode = nodeById.get(edge.target);
    });

    return { nodes, edges, nodeById };
}

function capVisibleSet(visible, data, hierarchy, capLimit) {
    const nodes = Array.from(visible).map((id) => {
        const node = data.nodeById.get(id);
        return {
            id,
            depth: hierarchy.depthById.get(id) || 0,
            path: node && node.file_path ? node.file_path : '',
            name: node && node.name ? node.name : ''
        };
    });

    nodes.sort((a, b) => {
        if (a.depth !== b.depth) return a.depth - b.depth;
        if (a.path !== b.path) return a.path.localeCompare(b.path);
        if (a.name !== b.name) return a.name.localeCompare(b.name);
        return String(a.id).localeCompare(String(b.id));
    });

    const keep = new Set();
    for (let i = 0; i < nodes.length && keep.size < capLimit; i += 1) {
        keep.add(nodes[i].id);
    }

    return keep;
}

function selectSeeds(data, hierarchy, levelKey) {
    const nodes = data.nodes;

    switch (levelKey) {
        case 'workspace': {
            let seeds = nodes.filter((node) => isWorkspaceKind(node) || isPackageKind(node));
            if (seeds.length === 0) {
                seeds = nodes.filter((node) => isTopLevelDirectory(node, hierarchy.depthById));
            }
            return seeds.map((node) => node.id);
        }
        case 'module': {
            let seeds = nodes.filter((node) => isModuleKind(node));
            if (seeds.length === 0) {
                seeds = nodes.filter((node) => isTopLevelDirectory(node, hierarchy.depthById));
            }
            return seeds.map((node) => node.id);
        }
        case 'directory':
            return nodes.filter((node) => rawKindKey(node) === 'directory').map((node) => node.id);
        case 'file':
            return nodes.filter((node) => rawKindKey(node) === 'file').map((node) => node.id);
        case 'symbol':
        default:
            return nodes.filter((node) => SYMBOL_KINDS.has(rawKindKey(node))).map((node) => node.id);
    }
}

function includeWithAncestors(id, visible, parentById) {
    let current = id;
    while (current !== undefined && current !== null) {
        if (!visible.has(current)) {
            visible.add(current);
        }
        const parent = parentById.get(current);
        if (!parent || parent === current) {
            break;
        }
        current = parent;
    }
}

function displayKindForLevel(node, levelKey, depth) {
    const kind = rawKindKey(node);
    if (depth === 0 && kind === 'directory') {
        return 'workspaceroot';
    }
    if (levelKey === 'workspace' || levelKey === 'module') {
        if (kind === 'directory' && depth === 1) {
            return 'module';
        }
    }
    return kind || 'unknown';
}

function buildContainsEdges(nodes, visible, parentById, nodeById) {
    const edges = [];
    const seen = new Set();

    nodes.forEach((node) => {
        const parent = nearestVisibleAncestor(node.id, visible, parentById);
        if (!parent || parent === node.id) {
            return;
        }
        const key = `${parent}-${node.id}`;
        if (seen.has(key)) {
            return;
        }
        seen.add(key);
        if (!nodeById.get(parent)) {
            return;
        }
        edges.push({
            id: `contains-${key}`,
            source: parent,
            target: node.id,
            kind: 'Contains',
            edge_source: 'Structural',
            confidence: 1,
            label: 'contains'
        });
    });

    return edges;
}

function buildAggregatedEdges(data, visible, parentById, nodeById) {
    const aggregated = new Map();

    data.edges.forEach((edge) => {
        if (isContainsEdge(edge.kind)) {
            return;
        }
        if (!data.nodeById.get(edge.source) || !data.nodeById.get(edge.target)) {
            return;
        }
        const source = nearestVisibleAncestor(edge.source, visible, parentById);
        const target = nearestVisibleAncestor(edge.target, visible, parentById);
        if (!source || !target || source === target) {
            return;
        }

        const key = `${source}-${target}`;
        let agg = aggregated.get(key);
        if (!agg) {
            agg = {
                source,
                target,
                count: 0,
                kind_counts: {},
                source_counts: {},
                min_confidence: null
            };
            aggregated.set(key, agg);
        }

        const kind = edge.kind || edge.label || 'Relation';
        const sourceType = edge.edge_source || edge.edgeSource || 'Structural';

        agg.count += 1;
        agg.kind_counts[kind] = (agg.kind_counts[kind] || 0) + 1;
        agg.source_counts[sourceType] = (agg.source_counts[sourceType] || 0) + 1;

        if (typeof edge.confidence === 'number') {
            if (agg.min_confidence === null || edge.confidence < agg.min_confidence) {
                agg.min_confidence = edge.confidence;
            }
        }
    });

    const edges = [];
    let index = 0;
    aggregated.forEach((agg) => {
        const dominantKind = dominantKey(agg.kind_counts) || 'Relation';
        const dominantSource = dominantKey(agg.source_counts) || 'Structural';
        const label = agg.count > 1 ? `${agg.count} relations` : dominantKind;

        edges.push({
            id: `agg-${agg.source}-${agg.target}-${index}`,
            source: agg.source,
            target: agg.target,
            kind: dominantKind,
            edge_source: dominantSource,
            confidence: agg.min_confidence,
            label,
            count: agg.count,
            kind_counts: agg.kind_counts,
            source_counts: agg.source_counts
        });
        index += 1;
    });

    return edges;
}

function dominantKey(counts) {
    let bestKey = null;
    let bestCount = -1;
    Object.keys(counts).forEach((key) => {
        if (counts[key] > bestCount) {
            bestKey = key;
            bestCount = counts[key];
        }
    });
    return bestKey;
}

function nearestVisibleAncestor(id, visible, parentById) {
    if (visible.has(id)) {
        return id;
    }
    let current = parentById.get(id);
    while (current) {
        if (visible.has(current)) {
            return current;
        }
        current = parentById.get(current);
    }
    return null;
}

function buildHierarchyCache(data) {
    const parentById = new Map();
    const childrenById = new Map();
    const directoryByPath = new Map();
    const fileByPath = new Map();

    data.nodes.forEach((node) => {
        const kind = rawKindKey(node);
        if (kind === 'directory') {
            if (node.file_path) {
                directoryByPath.set(node.file_path, node.id);
            }
        } else if (kind === 'file') {
            if (node.file_path) {
                fileByPath.set(node.file_path, node.id);
            }
        }
    });

    data.edges
        .filter((edge) => isContainsEdge(edge.kind))
        .forEach((edge) => {
            if (!parentById.has(edge.target)) {
                parentById.set(edge.target, edge.source);
            }
            addChild(childrenById, edge.source, edge.target);
        });

    data.nodes.forEach((node) => {
        if (parentById.has(node.id)) {
            return;
        }
        const kind = rawKindKey(node);
        const path = node.file_path;
        if (!path) {
            return;
        }

        if (kind !== 'directory') {
            const fileId = fileByPath.get(path);
            if (fileId && fileId !== node.id) {
                parentById.set(node.id, fileId);
                addChild(childrenById, fileId, node.id);
                return;
            }
        }

        const parentPath = path.split('/').slice(0, -1).join('/');
        const parentId = directoryByPath.get(parentPath);
        if (parentId && parentId !== node.id) {
            parentById.set(node.id, parentId);
            addChild(childrenById, parentId, node.id);
        }
    });

    const roots = data.nodes
        .filter((node) => !parentById.has(node.id))
        .map((node) => node.id);

    const depthById = new Map();
    const queue = roots.map((id) => ({ id, depth: 0 }));
    while (queue.length) {
        const { id, depth } = queue.shift();
        if (depthById.has(id)) {
            continue;
        }
        depthById.set(id, depth);
        const children = childrenById.get(id) || [];
        children.forEach((childId) => {
            queue.push({ id: childId, depth: depth + 1 });
        });
    }

    return { parentById, childrenById, roots, depthById, directoryByPath, fileByPath };
}

function addChild(childrenById, parentId, childId) {
    if (!childrenById.has(parentId)) {
        childrenById.set(parentId, []);
    }
    const children = childrenById.get(parentId);
    if (!children.includes(childId)) {
        children.push(childId);
    }
}

function normalizeGraph(graphData) {
    const nodes = (graphData.nodes || []).map((node) => {
        const id = toId(node.id);
        const kind = node.kind || 'Unknown';
        const filePath = node.file_path ? String(node.file_path).replace(/\\/g, '/') : '';
        return {
            ...node,
            id,
            kind,
            file_path: filePath,
            width: node.width || NODE_WIDTH,
            height: node.height || NODE_HEIGHT
        };
    });

    const nodeById = new Map(nodes.map((node) => [node.id, node]));

    const edges = (graphData.edges || []).map((edge, index) => {
        const rawSource = edge.source && typeof edge.source === 'object' ? edge.source.id : edge.source;
        const rawTarget = edge.target && typeof edge.target === 'object' ? edge.target.id : edge.target;
        const source = toId(rawSource);
        const target = toId(rawTarget);
        const kind = edge.kind || edge.label || 'Relation';
        return {
            ...edge,
            id: edge.id ? toId(edge.id) : `${source}-${target}-${kind}-${index}`,
            source,
            target,
            kind,
            sourceNode: nodeById.get(source),
            targetNode: nodeById.get(target)
        };
    }).filter((edge) => edge.sourceNode && edge.targetNode);

    return { nodes, edges, nodeById };
}

function layoutAndRender(data, { refit } = {}) {
    if (simulation) {
        simulation.stop();
        simulation = null;
    }

    if (layoutMode === 'force') {
        applyForceLayout(data);
    } else {
        applyHierarchyLayout(data);
    }

    drawGraph(data);

    if (refit) {
        fitToView();
    }

    updateGraphStatus();
    const emptyState = document.getElementById('empty-state');
    if (emptyState) {
        emptyState.style.display = data.nodes.length ? 'none' : 'block';
    }
}

function applyHierarchyLayout(data) {
    const hierarchy = buildHierarchyCache(data);
    const nodeWrappers = new Map();

    data.nodes.forEach((node) => {
        nodeWrappers.set(node.id, { data: node, children: [] });
    });

    hierarchy.childrenById.forEach((children, parentId) => {
        const parentWrapper = nodeWrappers.get(parentId);
        if (!parentWrapper) {
            return;
        }
        children.forEach((childId) => {
            const childWrapper = nodeWrappers.get(childId);
            if (childWrapper && childWrapper !== parentWrapper) {
                parentWrapper.children.push(childWrapper);
            }
        });
    });

    const roots = data.nodes
        .filter((node) => !hierarchy.parentById.has(node.id))
        .map((node) => nodeWrappers.get(node.id));

    const syntheticRoot = {
        data: {
            id: '__root__',
            name: 'Workspace',
            kind: 'WorkspaceRoot',
            width: NODE_WIDTH + 30,
            height: NODE_HEIGHT + 10
        },
        children: roots
    };

    const hierarchyTree = d3.hierarchy(syntheticRoot, (d) => d.children);
    const maxDepth = d3.max(hierarchyTree.descendants(), (node) => node.depth) || 1;
    const radiusStep = Math.max(NODE_WIDTH, NODE_HEIGHT) * 2.4;
    const radius = Math.max(1, maxDepth) * radiusStep;

    const treeLayout = d3.tree().size([2 * Math.PI, radius]);
    treeLayout(hierarchyTree);

    hierarchyTree.descendants().forEach((node) => {
        if (node.data && node.data.data) {
            const angle = node.x - Math.PI / 2;
            node.data.data.x = Math.cos(angle) * node.y;
            node.data.data.y = Math.sin(angle) * node.y;
            node.data.data.depth = node.depth;
        }
    });
}

function applyForceLayout(data) {
    const size = getSvgSize();
    const baseRadius = Math.min(size.width, size.height) / 2;

    data.nodes.forEach((node, index) => {
        if (!Number.isFinite(node.x) || !Number.isFinite(node.y)) {
            const angle = index * 0.618 * Math.PI * 2;
            const radius = Math.sqrt(index + 1) * 14;
            node.x = Math.cos(angle) * radius;
            node.y = Math.sin(angle) * radius;
        }
    });

    simulation = d3.forceSimulation(data.nodes)
        .force('link', d3.forceLink(data.edges)
            .id((d) => d.id)
            .distance(140)
            .strength(0.9)
        )
        .force('charge', d3.forceManyBody().strength(-380))
        .force('center', d3.forceCenter(0, 0))
        .force('radial', d3.forceRadial(baseRadius * 0.5, 0, 0).strength(0.05))
        .force('collision', d3.forceCollide().radius((d) => Math.max(d.width, d.height) / 2 + 16))
        .on('tick', () => positionGraph());
}

function drawGraph(data) {
    const edgeSelection = edgesGroup
        .selectAll('path.edge')
        .data(data.edges, (d) => d.id);

    edgeSelection.exit().remove();

    const edgeEnter = edgeSelection
        .enter()
        .append('path')
        .attr('class', (d) => `edge ${edgeCategory(d)}`)
        .attr('marker-end', (d) => `url(#${edgeMarker(d)})`);

    edgeEnter.append('title')
        .text((d) => edgeTitle(d));

    const edges = edgeEnter.merge(edgeSelection);

    const nodeSelection = nodesGroup
        .selectAll('g.node')
        .data(data.nodes, (d) => d.id);

    nodeSelection.exit().remove();

    const nodeEnter = nodeSelection
        .enter()
        .append('g')
        .attr('class', (d) => nodeClass(d))
        .call(d3.drag()
            .on('start', dragstarted)
            .on('drag', dragged)
            .on('end', dragended)
        );

    nodeEnter.append('rect')
        .attr('x', (d) => -d.width / 2)
        .attr('y', (d) => -d.height / 2)
        .attr('width', (d) => d.width)
        .attr('height', (d) => d.height)
        .attr('rx', 10)
        .attr('ry', 10);

    nodeEnter.append('text')
        .attr('text-anchor', 'middle')
        .attr('dy', 4)
        .text((d) => truncateLabel(labelForNode(d), 18));

    nodeEnter.append('title')
        .text((d) => `${labelForNode(d)}\n${d.file_path || ''}`);

    const nodes = nodeEnter.merge(nodeSelection)
        .attr('class', (d) => nodeClass(d))
        .on('click', (event, d) => {
            event.stopPropagation();
            showNodeDetails(d);
            highlightNeighbors(d, data);
        });

    svg.on('click', () => clearHighlights());

    positionGraph(edges, nodes);
    applyFilters();

    if (searchQuery) {
        searchNodes(searchQuery);
    }
}

function nodeClass(node) {
    const kind = displayKindKey(node) || 'unknown';
    return `node kind-${kind} group-${nodeGroup(node)}`;
}

function positionGraph(edgesSelection, nodesSelection) {
    const nodes = nodesSelection || nodesGroup.selectAll('g.node');
    const edges = edgesSelection || edgesGroup.selectAll('path.edge');

    nodes.attr('transform', (d) => {
        const pos = nodePosition(d);
        return `translate(${pos.x}, ${pos.y})`;
    });

    edges.attr('d', (d) => edgePath(d));
}

function nodePosition(node) {
    return { x: node.x || 0, y: node.y || 0 };
}

function edgePath(edge) {
    const source = edge.sourceNode;
    const target = edge.targetNode;
    if (!source || !target) {
        return '';
    }

    return `M${source.x},${source.y} L${target.x},${target.y}`;
}

function dragstarted(event, d) {
    if (simulation && !event.active) {
        simulation.alphaTarget(0.3).restart();
    }
    d.fx = d.x;
    d.fy = d.y;
}

function dragged(event, d) {
    d.fx = event.x;
    d.fy = event.y;
    if (!simulation) {
        d.x = event.x;
        d.y = event.y;
        positionGraph();
    }
}

function dragended(event, d) {
    if (simulation && !event.active) {
        simulation.alphaTarget(0);
    }
    d.fx = null;
    d.fy = null;
}

function applyDiff(_diff) {
    if (!currentData) {
        return;
    }

    if (window.currentGraphData) {
        renderGraph(window.currentGraphData);
    }
}

function applyFilters() {
    if (!currentView) {
        return;
    }

    const showDirectories = isChecked('filter-directories');
    const showFiles = isChecked('filter-files');
    const showSymbols = isChecked('filter-symbols');
    const showContains = isChecked('filter-edges-contains');
    const showSyntactic = isChecked('filter-edges-syntactic');
    const showSemantic = isChecked('filter-edges-semantic');
    const showOther = isChecked('filter-edges-other');

    const visibleNodes = new Set(
        currentView.nodes
            .filter((node) => {
                const category = nodeGroup(node);
                if (category === 'directory') return showDirectories;
                if (category === 'file') return showFiles;
                return showSymbols;
            })
            .map((node) => node.id)
    );

    nodesGroup.selectAll('g.node')
        .classed('is-hidden', (d) => !visibleNodes.has(d.id));

    edgesGroup.selectAll('path.edge')
        .classed('is-hidden', (edge) => {
            if (!visibleNodes.has(edge.source) || !visibleNodes.has(edge.target)) {
                return true;
            }
            const category = edgeCategory(edge);
            if (category === 'contains') return !showContains;
            if (category === 'semantic') return !showSemantic;
            if (category === 'syntactic') return !showSyntactic;
            return !showOther;
        });
}

function searchNodes(query) {
    searchQuery = (query || '').trim().toLowerCase();

    if (!currentView) {
        return;
    }

    if (!searchQuery) {
        clearSearchHighlights();
        updateGraphStatus();
        return;
    }

    const matches = new Set(
        currentView.nodes
            .filter((node) => {
                const name = (node.name || '').toLowerCase();
                const path = (node.file_path || '').toLowerCase();
                const qualified = (node.qualified_name || '').toLowerCase();
                return name.includes(searchQuery) || path.includes(searchQuery) || qualified.includes(searchQuery);
            })
            .map((node) => node.id)
    );

    nodesGroup.selectAll('g.node')
        .classed('is-match', (d) => matches.has(d.id))
        .classed('is-dimmed', (d) => !matches.has(d.id));

    updateGraphStatus(`Matches: ${matches.size}`);
}

function clearSearchHighlights() {
    nodesGroup.selectAll('g.node')
        .classed('is-match', false)
        .classed('is-dimmed', false);
}

function highlightNeighbors(node, data) {
    if (!data) {
        return;
    }

    const neighborIds = new Set([node.id]);
    const highlightEdges = new Set();

    data.edges.forEach((edge) => {
        if (edge.source === node.id || edge.target === node.id) {
            neighborIds.add(edge.source);
            neighborIds.add(edge.target);
            highlightEdges.add(edge.id);
        }
    });

    nodesGroup.selectAll('g.node')
        .classed('is-dimmed', (d) => !neighborIds.has(d.id))
        .classed('is-match', (d) => neighborIds.has(d.id));

    edgesGroup.selectAll('path.edge')
        .classed('is-highlighted', (edge) => highlightEdges.has(edge.id));
}

function clearHighlights() {
    clearSearchHighlights();
    edgesGroup.selectAll('path.edge')
        .classed('is-highlighted', false);
}

function resetView() {
    clearHighlights();
    const searchInput = document.getElementById('search-input');
    if (searchInput) {
        searchInput.value = '';
    }
    searchQuery = '';
    fitToView();
}

function fitToView() {
    if (!currentView || currentView.nodes.length === 0) {
        return;
    }

    const { width, height } = getSvgSize();
    const bounds = getBounds(currentView.nodes);
    const graphWidth = bounds.maxX - bounds.minX + 120;
    const graphHeight = bounds.maxY - bounds.minY + 120;

    const scale = Math.min(width / graphWidth, height / graphHeight, 1);
    const translateX = width / 2 - scale * (bounds.minX + bounds.maxX) / 2;
    const translateY = height / 2 - scale * (bounds.minY + bounds.maxY) / 2;

    suppressZoomUpdates = true;
    svg.transition()
        .duration(500)
        .on('end', () => {
            suppressZoomUpdates = false;
        })
        .call(zoomBehavior.transform, d3.zoomIdentity.translate(translateX, translateY).scale(scale));
}

function getBounds(nodes) {
    let minX = Infinity;
    let maxX = -Infinity;
    let minY = Infinity;
    let maxY = -Infinity;

    nodes.forEach((node) => {
        const pos = nodePosition(node);
        const halfWidth = (node.width || NODE_WIDTH) / 2;
        const halfHeight = (node.height || NODE_HEIGHT) / 2;
        minX = Math.min(minX, pos.x - halfWidth);
        maxX = Math.max(maxX, pos.x + halfWidth);
        minY = Math.min(minY, pos.y - halfHeight);
        maxY = Math.max(maxY, pos.y + halfHeight);
    });

    return { minX, maxX, minY, maxY };
}

function showNodeDetails(node) {
    const details = document.getElementById('node-details');
    if (!details) {
        return;
    }

    const summary = getAiSummary(node);
    const summaryBlock = summary
        ? `<div class="summary">${escapeHtml(summary)}</div>`
        : '<div class="empty">No AI summary available yet.</div>';

    const metadata = buildMetadataDisplay(node.metadata || {});

    details.innerHTML = `
        <h3>Details</h3>
        <div><strong>${escapeHtml(node.name || node.qualified_name || 'Unnamed')}</strong></div>
        <div>Kind: ${escapeHtml(node.kind || 'Unknown')}</div>
        <div>Path: ${escapeHtml(node.file_path || 'N/A')}</div>
        ${node.language ? `<div>Language: ${escapeHtml(node.language)}</div>` : ''}
        ${node.line_start ? `<div>Lines: ${node.line_start}-${node.line_end || node.line_start}</div>` : ''}
        <div style="margin-top: 12px;">AI Summary:</div>
        ${summaryBlock}
        <div style="margin-top: 12px;">Metadata:</div>
        ${metadata}
    `;
}

function buildMetadataDisplay(metadata) {
    const cleaned = { ...metadata };
    delete cleaned.ai_summary;
    delete cleaned.summary;
    delete cleaned.aiSummary;
    delete cleaned.blurb;

    const keys = Object.keys(cleaned);
    if (keys.length === 0) {
        return '<div class="empty">No metadata</div>';
    }

    return `<pre>${escapeHtml(JSON.stringify(cleaned, null, 2))}</pre>`;
}

function getAiSummary(node) {
    const metadata = node.metadata || {};
    const summary = node.summary || node.ai_summary || metadata.ai_summary || metadata.summary || metadata.aiSummary || metadata.blurb;
    if (summary) {
        return summary;
    }
    return fallbackSummary(node);
}

function fallbackSummary(node) {
    const kind = displayKindKey(node);
    const name = node.name || node.qualified_name || 'This item';

    if (kind === 'function' || kind === 'method') {
        return `${name} encapsulates executable behavior and participates in runtime flows.`;
    }
    if (kind === 'class' || kind === 'struct' || kind === 'interface') {
        return `${name} defines a reusable type boundary and its associated responsibilities.`;
    }
    if (kind === 'module' || kind === 'package') {
        return `${name} groups related files into a cohesive module boundary.`;
    }
    if (kind === 'file') {
        return `${name} hosts related definitions and wires them together.`;
    }
    if (kind === 'directory') {
        return `${name} contains a scoped slice of the codebase.`;
    }
    return `${name} represents a ${kind || 'code'} element within the system.`;
}

function labelForNode(node) {
    if (node.name) return node.name;
    if (node.qualified_name) return node.qualified_name;
    if (node.file_path) {
        const parts = node.file_path.split('/');
        return parts[parts.length - 1];
    }
    return node.id;
}

function truncateLabel(text, maxLength) {
    if (!text) return '';
    if (text.length <= maxLength) return text;
    return `${text.slice(0, maxLength - 3)}...`;
}

function nodeGroup(node) {
    const kind = displayKindKey(node);
    if (kind === 'file') {
        return 'file';
    }
    if (STRUCTURAL_KINDS.has(kind)) {
        return 'directory';
    }
    return 'symbol';
}

function edgeCategory(edge) {
    const kind = (edge.kind || '').toString().toLowerCase();
    if (isContainsEdge(kind)) {
        return 'contains';
    }
    const source = (edge.edge_source || edge.edgeSource || '').toString().toLowerCase();
    if (source === 'ai') {
        return 'semantic';
    }
    if (source === 'structural' || source === 'heuristic') {
        return 'syntactic';
    }
    if (['calls', 'dependson', 'uses', 'imports', 'implements', 'inherits', 'semanticreference', 'configures'].includes(kind)) {
        return 'semantic';
    }
    return 'other';
}

function edgeMarker(edge) {
    const category = edgeCategory(edge);
    if (category === 'contains') return 'arrow-contains';
    if (category === 'semantic') return 'arrow-semantic';
    if (category === 'syntactic') return 'arrow-syntactic';
    return 'arrow-other';
}

function edgeTitle(edge) {
    const count = edge.count ? `${edge.count} relations` : null;
    const kind = edge.kind ? `Kind: ${edge.kind}` : null;
    const source = edge.edge_source ? `Source: ${edge.edge_source}` : null;
    return [count, kind, source].filter(Boolean).join('\n');
}

function isContainsEdge(kind) {
    return (kind || '').toString().toLowerCase().includes('contain');
}

function rawKindKey(node) {
    return (node.kind || '').toString().toLowerCase();
}

function displayKindKey(node) {
    return (node.displayKind || node.kind || '').toString().toLowerCase();
}

function isWorkspaceKind(node) {
    return rawKindKey(node) === 'workspaceroot';
}

function isPackageKind(node) {
    return rawKindKey(node) === 'package';
}

function isModuleKind(node) {
    return rawKindKey(node) === 'module';
}

function isTopLevelDirectory(node, depthById) {
    return rawKindKey(node) === 'directory' && (depthById.get(node.id) || 0) === 1;
}

function toId(value) {
    if (value === null || value === undefined) return '';
    return typeof value === 'string' ? value : value.toString();
}

function isChecked(id) {
    const element = document.getElementById(id);
    return element ? element.checked : true;
}

function getSvgSize() {
    const rect = svg.node().getBoundingClientRect();
    return { width: rect.width || 1, height: rect.height || 1 };
}

function updateGraphStatus(extra) {
    const levelName = LEVELS[currentLevel]?.name || 'Unknown';
    const base = `Level: ${levelName} | Nodes: ${currentView ? currentView.nodes.length : 0}/${currentData ? currentData.nodes.length : 0} | Edges: ${currentView ? currentView.edges.length : 0}`;
    const capped = currentView && currentView.isCapped ? ' | Capped' : '';
    updateStatus(`${extra ? `${extra} | ` : ''}${base}${capped}`);
}

function updateStatus(text) {
    const status = document.getElementById('status');
    if (status) {
        status.textContent = text;
    }
}

function escapeHtml(value) {
    return String(value)
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#39;');
}

// Initialize on page load
window.addEventListener('DOMContentLoaded', init);

// Export for protocol.js
window.renderGraph = renderGraph;
