// Graph visualization using D3.js with force simulation

let graph = null;
let svg = null;
let simulation = null;
let ws = null;
let currentData = null;
let visibleNodes = [];
let visibleEdges = [];
let currentTransform = d3.zoomIdentity;

// Default node dimensions
const NODE_WIDTH = 120;
const NODE_HEIGHT = 40;

// Performance optimization settings
const MAX_VISIBLE_NODES = 500; // Maximum nodes to render at once
const VIEWPORT_PADDING = 100; // Extra padding around viewport for smooth panning

// Initialize the visualization
function init() {
    svg = d3.select('#graph');

    // Set up zoom behavior with viewport update
    const zoom = d3.zoom()
        .scaleExtent([0.1, 10])
        .on('zoom', (event) => {
            currentTransform = event.transform;
            svg.select('g').attr('transform', event.transform);
            // Throttle viewport updates for performance
            if (!updateVisibility.timeout) {
                updateVisibility.timeout = setTimeout(() => {
                    updateVisibility();
                    updateVisibility.timeout = null;
                }, 100);
            }
        });

    svg.call(zoom);

    // Create main group for graph elements
    svg.append('g').attr('id', 'graph-group');

    // Add arrow marker definition for edges
    svg.append('defs').append('marker')
        .attr('id', 'arrowhead')
        .attr('viewBox', '-0 -5 10 10')
        .attr('refX', 20)
        .attr('refY', 0)
        .attr('orient', 'auto')
        .attr('markerWidth', 8)
        .attr('markerHeight', 8)
        .append('path')
        .attr('d', 'M 0,-5 L 10,0 L 0,5')
        .attr('fill', '#666');

    // Connect to WebSocket
    connectWebSocket();
}

// Connect to WebSocket for real-time updates
function connectWebSocket() {
    // Use the WebSocket protocol from protocol.js if available
    if (window.WebSocketProtocol) {
        console.log('Using WebSocketProtocol for connection');
        window.WebSocketProtocol.connect();
    } else {
        // Fallback to direct WebSocket connection
        const wsUrl = `ws://${window.location.hostname}:7890/ws`;
        ws = new WebSocket(wsUrl);
        
        ws.onopen = () => {
            console.log('Connected to Canopy server');
            document.getElementById('status').textContent = 'Connected';
        };
        
        ws.onmessage = (event) => {
            const data = JSON.parse(event.data);
            if (data.type === 'graph_diff') {
                applyDiff(data.diff);
            } else if (data.type === 'full_graph') {
                renderGraph(data.graph);
            }
        };
        
        ws.onerror = (error) => {
            console.error('WebSocket error:', error);
            document.getElementById('status').textContent = 'Connection error';
        };
        
        ws.onclose = () => {
            console.log('Disconnected from Canopy server');
            document.getElementById('status').textContent = 'Disconnected';
            setTimeout(connectWebSocket, 3000); // Reconnect after 3s
        };
    }
}

// Render the full graph with D3 force simulation
function renderGraph(graphData) {
    currentData = graphData;

    // Stop any existing simulation
    if (simulation) {
        simulation.stop();
    }

    // Clear existing elements
    svg.select('#graph-group').selectAll('*').remove();

    // Process nodes - add default dimensions and initialize positions
    const nodes = (graphData.nodes || []).map((node, i) => ({
        ...node,
        width: node.width || NODE_WIDTH,
        height: node.height || NODE_HEIGHT,
        // Initialize position if not present (spread nodes out initially)
        x: node.x !== undefined ? node.x : (i % 10) * 150 + 100,
        y: node.y !== undefined ? node.y : Math.floor(i / 10) * 100 + 100
    }));

    // Create a map for quick node lookup by id
    const nodeMap = new Map(nodes.map(n => [n.id, n]));

    // Process edges - convert source/target from IDs to node references
    const edges = (graphData.edges || []).map(edge => ({
        ...edge,
        source: typeof edge.source === 'string' ? nodeMap.get(edge.source) || edge.source : edge.source,
        target: typeof edge.target === 'string' ? nodeMap.get(edge.target) || edge.target : edge.target
    })).filter(edge => edge.source && edge.target);

    // Create groups for edges and nodes (edges first so they render behind nodes)
    const edgesGroup = svg.select('#graph-group').append('g').attr('class', 'edges');
    const nodesGroup = svg.select('#graph-group').append('g').attr('class', 'nodes');

    // Create node elements
    const nodeElements = nodesGroup.selectAll('.node')
        .data(nodes)
        .enter()
        .append('g')
        .attr('class', 'node')
        .call(d3.drag()
            .on('start', dragstarted)
            .on('drag', dragged)
            .on('end', dragended));

    // Add rectangles for nodes
    nodeElements.append('rect')
        .attr('class', d => `node-${(d.kind || 'default').toLowerCase()}`)
        .attr('x', d => -d.width / 2)
        .attr('y', d => -d.height / 2)
        .attr('width', d => d.width)
        .attr('height', d => d.height)
        .attr('rx', 4);

    // Add labels to nodes
    nodeElements.append('text')
        .attr('class', 'node-label')
        .text(d => d.name || d.id);

    // Add click handlers
    nodeElements.on('click', function(event, d) {
        event.stopPropagation();
        showNodeDetails(d);
    });

    // Create edge elements
    const edgeElements = edgesGroup.selectAll('.edge')
        .data(edges)
        .enter()
        .append('g')
        .attr('class', 'edge');

    const edgePaths = edgeElements.append('path')
        .attr('marker-end', 'url(#arrowhead)');

    // Add edge labels
    const edgeLabels = edgeElements.append('text')
        .attr('class', 'edge-label')
        .text(d => d.label || d.kind || '');

    // Get SVG dimensions for centering
    const svgRect = svg.node().getBoundingClientRect();
    const centerX = svgRect.width / 2;
    const centerY = svgRect.height / 2;

    // Create force simulation
    simulation = d3.forceSimulation(nodes)
        .force('link', d3.forceLink(edges)
            .id(d => d.id)
            .distance(150))
        .force('charge', d3.forceManyBody()
            .strength(-400))
        .force('center', d3.forceCenter(centerX, centerY))
        .force('collision', d3.forceCollide()
            .radius(d => Math.max(d.width, d.height) / 2 + 20))
        .on('tick', ticked);

    // Update positions on each tick
    function ticked() {
        nodeElements.attr('transform', d => `translate(${d.x}, ${d.y})`);

        edgePaths.attr('d', d => {
            const sourceX = d.source.x;
            const sourceY = d.source.y;
            const targetX = d.target.x;
            const targetY = d.target.y;
            return `M${sourceX},${sourceY} L${targetX},${targetY}`;
        });

        edgeLabels
            .attr('x', d => (d.source.x + d.target.x) / 2)
            .attr('y', d => (d.source.y + d.target.y) / 2);
        
        // Update visibility during animation
        if (simulation.alpha() < 0.5) {
            updateVisualization();
        }
    }

    // Drag functions
    function dragstarted(event, d) {
        if (!event.active) simulation.alphaTarget(0.3).restart();
        d.fx = d.x;
        d.fy = d.y;
    }

    function dragged(event, d) {
        d.fx = event.x;
        d.fy = event.y;
    }

    function dragended(event, d) {
        if (!event.active) simulation.alphaTarget(0);
        d.fx = null;
        d.fy = null;
    }
}

// Apply incremental diff to existing graph
function applyDiff(diff) {
    console.log('Applying diff:', diff);
    
    // For now, just re-render the full graph
    // In future, implement incremental updates
    if (currentData) {
        renderGraph(currentData);
    }
}

// Show node details in sidebar
function showNodeDetails(node) {
    const detailsDiv = document.getElementById('node-details');
    detailsDiv.innerHTML = `
        <h4>${node.name}</h4>
        <p><strong>Type:</strong> ${node.kind}</p>
        <p><strong>Path:</strong> ${node.file_path}</p>
        ${node.language ? `<p><strong>Language:</strong> ${node.language}</p>` : ''}
        ${node.line_start ? `<p><strong>Lines:</strong> ${node.line_start}-${node.line_end}</p>` : ''}
        ${node.metadata && Object.keys(node.metadata).length > 0 ? 
            `<p><strong>Metadata:</strong></p>
            <pre>${JSON.stringify(node.metadata, null, 2)}</pre>` : ''}
    `;
}

// Search functionality
function searchNodes(query) {
    if (!currentData || !query) return;
    
    const lowerQuery = query.toLowerCase();
    const matchingNodes = currentData.nodes.filter(node => 
        node.name.toLowerCase().includes(lowerQuery) ||
        node.file_path.toLowerCase().includes(lowerQuery)
    );
    
    // Highlight matching nodes
    svg.selectAll('.node')
        .classed('highlighted', d => matchingNodes.some(n => n.id === d.id))
        .classed('dimmed', d => !matchingNodes.some(n => n.id === d.id));
    
    // Update status
    document.getElementById('status').textContent = 
        `Found ${matchingNodes.length} matching nodes for "${query}"`;
}

// Handle filter changes
function updateFilters() {
    const filters = {
        directory: document.getElementById('filter-directories').checked,
        file: document.getElementById('filter-files').checked,
        function: document.getElementById('filter-functions').checked
    };
    
    // Apply filters to nodes
    svg.selectAll('.node')
        .style('display', d => {
            const nodeKind = d.kind?.toLowerCase();
            if (nodeKind === 'directory' && !filters.directory) return 'none';
            if (nodeKind === 'file' && !filters.file) return 'none';
            if (nodeKind === 'function' && !filters.function) return 'none';
            return 'block';
        });
    
    // Hide edges connected to hidden nodes
    svg.selectAll('.edge')
        .style('display', function(d) {
            const sourceVisible = d3.select(this).datum().source.style('display') !== 'none';
            const targetVisible = d3.select(this).datum().target.style('display') !== 'none';
            return (sourceVisible && targetVisible) ? 'block' : 'none';
        });
}

// Control functions
function resetZoom() {
    svg.transition()
        .duration(750)
        .call(d3.zoom().transform, d3.zoomIdentity);
}

function toggleAnimation() {
    // Toggle animation on edges
    const edges = svg.selectAll('.edge');
    edges.classed('animated', !edges.classed('animated'));
}

// Check if a node is within the viewport
function isNodeInViewport(node, transform, viewport) {
    const x = (node.x || 0) * transform.k + transform.x;
    const y = (node.y || 0) * transform.k + transform.y;
    return x >= -VIEWPORT_PADDING && 
           y >= -VIEWPORT_PADDING && 
           x <= viewport.width + VIEWPORT_PADDING && 
           y <= viewport.height + VIEWPORT_PADDING;
}

// Filter nodes based on viewport and performance limits
function filterVisibleNodes(nodes, transform, viewport) {
    // First, get nodes in viewport
    let inViewport = nodes.filter(node => isNodeInViewport(node, transform, viewport));
    
    // If too many nodes, prioritize important ones
    if (inViewport.length > MAX_VISIBLE_NODES) {
        // Sort by importance: directories first, then files, then symbols
        inViewport.sort((a, b) => {
            const importanceOrder = {
                'directory': 0,
                'file': 1,
                'module': 2,
                'class': 3,
                'function': 4,
                'method': 5,
                'default': 6
            };
            const aImportance = importanceOrder[a.kind?.toLowerCase()] ?? importanceOrder.default;
            const bImportance = importanceOrder[b.kind?.toLowerCase()] ?? importanceOrder.default;
            return aImportance - bImportance;
        });
        
        // Keep only the most important nodes
        inViewport = inViewport.slice(0, MAX_VISIBLE_NODES);
    }
    
    return inViewport;
}

// Filter edges to only show those between visible nodes
function filterVisibleEdges(edges, visibleNodeSet) {
    return edges.filter(edge => {
        const sourceVisible = visibleNodeSet.has(edge.source.id || edge.source);
        const targetVisible = visibleNodeSet.has(edge.target.id || edge.target);
        return sourceVisible && targetVisible;
    });
}

// Update visible nodes based on current viewport
function updateVisibility() {
    if (!currentData) return;
    
    const viewport = svg.node().getBoundingClientRect();
    visibleNodes = filterVisibleNodes(currentData.nodes, currentTransform, viewport);
    const visibleNodeSet = new Set(visibleNodes.map(n => n.id));
    visibleEdges = filterVisibleEdges(currentData.edges, visibleNodeSet);
    
    // Update the visualization
    updateVisualization();
    
    // Update status
    document.getElementById('status').textContent = 
        `Showing ${visibleNodes.length}/${currentData.nodes.length} nodes`;
}

// Update visualization with filtered data
function updateVisualization() {
    if (!simulation) return;
    
    // Update node visibility
    svg.selectAll('.node')
        .style('display', d => visibleNodes.includes(d) ? 'block' : 'none');
    
    // Update edge visibility
    svg.selectAll('.edge')
        .style('display', d => visibleEdges.includes(d) ? 'block' : 'none');
}

// Initialize on page load
document.addEventListener('DOMContentLoaded', init);

// Setup search and filter event listeners
document.addEventListener('DOMContentLoaded', () => {
    const searchInput = document.getElementById('search-input');
    const searchButton = document.getElementById('search-button');
    
    if (searchInput && searchButton) {
        searchButton.addEventListener('click', () => searchNodes(searchInput.value));
        searchInput.addEventListener('keypress', (e) => {
            if (e.key === 'Enter') searchNodes(searchInput.value);
        });
    }
    
    // Setup filter checkboxes
    ['filter-directories', 'filter-files', 'filter-functions'].forEach(id => {
        const checkbox = document.getElementById(id);
        if (checkbox) {
            checkbox.addEventListener('change', updateFilters);
        }
    });
});