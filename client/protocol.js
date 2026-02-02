// WebSocket protocol for real-time graph updates

let ws = null;
let reconnectTimeout = null;
let reconnectAttempts = 0;
const maxReconnectAttempts = 5;

// Connect to WebSocket server
function connectWebSocket() {
    const wsUrl = `ws://${window.location.hostname}:7890/ws`;
    
    ws = new WebSocket(wsUrl);
    
    ws.onopen = () => {
        console.log('Connected to Canopy WebSocket');
        updateStatus('Connected');
        reconnectAttempts = 0;
        
        // Request full graph on connection
        requestFullGraph();
    };
    
    ws.onmessage = (event) => {
        try {
            const message = JSON.parse(event.data);
            handleMessage(message);
        } catch (error) {
            console.error('Error parsing WebSocket message:', error);
        }
    };
    
    ws.onerror = (error) => {
        console.error('WebSocket error:', error);
        updateStatus('Connection error');
    };
    
    ws.onclose = () => {
        console.log('WebSocket connection closed');
        updateStatus('Disconnected');
        scheduleReconnect();
    };
}

// Handle incoming WebSocket messages
function handleMessage(message) {
    switch (message.type) {
        case 'graph_diff':
            handleGraphDiff(message.diff);
            break;
        case 'full_graph':
            handleFullGraph(message.graph);
            break;
        case 'error':
            handleError(message.error);
            break;
        default:
            console.warn('Unknown message type:', message.type);
    }
}

// Handle graph diff updates
function handleGraphDiff(diff) {
    console.log('Received graph diff:', diff);
    
    // Apply diff to current graph
    if (window.currentGraphData) {
        applyDiffToGraph(window.currentGraphData, diff);
        renderGraph(window.currentGraphData);
    }
}

// Handle full graph data
function handleFullGraph(graph) {
    console.log('Received full graph:', graph);
    window.currentGraphData = graph;
    renderGraph(graph);
}

// Handle errors
function handleError(error) {
    console.error('Server error:', error);
    updateStatus(`Error: ${error}`);
}

// Request full graph from server
function requestFullGraph() {
    if (ws && ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify({
            type: 'request_full_graph'
        }));
    }
}

// Send diff acknowledgment
function sendDiffAck(sequence) {
    if (ws && ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify({
            type: 'diff_ack',
            sequence: sequence
        }));
    }
}

// Apply diff to current graph data
function applyDiffToGraph(currentGraph, diff) {
    // For now, just update the sequence number
    // In future, implement proper incremental updates
    currentGraph.sequence = diff.sequence;
    
    // Mark changed nodes
    if (diff.modified_nodes) {
        diff.modified_nodes.forEach(nodeId => {
            const node = currentGraph.nodes.find(n => n.id === nodeId);
            if (node) {
                node.changed = true;
            }
        });
    }
}

// Update status display
function updateStatus(text) {
    const statusElement = document.getElementById('status');
    if (statusElement) {
        statusElement.textContent = text;
    }
}

// Schedule reconnection
function scheduleReconnect() {
    if (reconnectAttempts >= maxReconnectAttempts) {
        console.error('Max reconnection attempts reached');
        updateStatus('Connection failed');
        return;
    }
    
    reconnectAttempts++;
    const delay = Math.min(1000 * Math.pow(2, reconnectAttempts), 30000); // Exponential backoff
    
    console.log(`Scheduling reconnection in ${delay}ms (attempt ${reconnectAttempts})`);
    updateStatus(`Reconnecting... (${reconnectAttempts})`);
    
    reconnectTimeout = setTimeout(() => {
        console.log('Attempting reconnection...');
        connectWebSocket();
    }, delay);
}

// Disconnect WebSocket
function disconnectWebSocket() {
    if (reconnectTimeout) {
        clearTimeout(reconnectTimeout);
        reconnectTimeout = null;
    }
    
    if (ws) {
        ws.close();
        ws = null;
    }
}

// Export functions for use in graph.js
window.WebSocketProtocol = {
    connect: connectWebSocket,
    disconnect: disconnectWebSocket,
    requestFullGraph: requestFullGraph,
    sendDiffAck: sendDiffAck
};