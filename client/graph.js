// Graph visualization using D3.js and ELK.js

let graph = null;
let svg = null;
let simulation = null;
let ws = null;
let currentData = null;

// Initialize the visualization
function init() {
    svg = d3.select('#graph');
    
    // Set up zoom behavior
    const zoom = d3.zoom()
        .scaleExtent([0.1, 10])
        .on('zoom', (event) => {
            svg.select('g').attr('transform', event.transform);
        });
    
    svg.call(zoom);
    
    // Create main group for graph elements
    svg.append('g').attr('id', 'graph-group');
    
    // Connect to WebSocket
    connectWebSocket();
}

// Connect to WebSocket for real-time updates
function connectWebSocket() {
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

// Render the full graph
function renderGraph(graphData) {
    currentData = graphData;
    
    // Clear existing elements
    svg.select('#graph-group').selectAll('*').remove();
    
    // Create groups for nodes and edges
    const nodesGroup = svg.select('#graph-group').append('g').attr('class', 'nodes');
    const edgesGroup = svg.select('#graph-group').append('g').attr('class', 'edges');
    
    // Process nodes
    const nodes = graphData.nodes || [];
    const edges = graphData.edges || [];
    
    // Create node elements
    const nodeElements = nodesGroup.selectAll('.node')
        .data(nodes)
        .enter()
        .append('g')
        .attr('class', 'node')
        .attr('transform', d => `translate(${d.x}, ${d.y})`);
    
    // Add rectangles for nodes
    nodeElements.append('rect')
        .attr('class', d => `node-${d.kind.toLowerCase()}`)
        .attr('x', d => -d.width / 2)
        .attr('y', d => -d.height / 2)
        .attr('width', d => d.width)
        .attr('height', d => d.height)
        .attr('rx', 4);
    
    // Add labels to nodes
    nodeElements.append('text')
        .attr('class', 'node-label')
        .text(d => d.name);
    
    // Add click handlers
    nodeElements.on('click', function(event, d) {
        showNodeDetails(d);
    });
    
    // Create edge elements
    const edgeElements = edgesGroup.selectAll('.edge')
        .data(edges)
        .enter()
        .append('g')
        .attr('class', 'edge');
    
    edgeElements.append('path')
        .attr('d', d => {
            if (d.points && d.points.length >= 2) {
                const points = d.points.map(p => `${p.x},${p.y}`).join(' ');
                return `M${d.source.x},${d.source.y} L${points} L${d.target.x},${d.target.y}`;
            }
            return `M${d.source.x},${d.source.y} L${d.target.x},${d.target.y}`;
        });
    
    // Add edge labels
    edgeElements.append('text')
        .attr('class', 'edge-label')
        .attr('x', d => (d.source.x + d.target.x) / 2)
        .attr('y', d => (d.source.y + d.target.y) / 2)
        .text(d => d.label || '');
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

// Initialize on page load
document.addEventListener('DOMContentLoaded', init);