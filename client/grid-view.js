// Grid-based hierarchical visualization for Canopy

let currentGraph = null;
let currentView = 'hierarchical'; // 'hierarchical' or 'flat'
let currentPath = [];
let selectedModule = null;

// Initialize the grid view
function initGridView() {
    // Connect to WebSocket
    if (window.WebSocketProtocol) {
        window.WebSocketProtocol.connect();
    } else {
        connectWebSocket();
    }
    
    // Set up event listeners
    document.getElementById('search').addEventListener('input', handleSearch);
    
    // Set up filter listeners
    const filters = ['directories', 'files', 'functions', 'classes', 'ai'];
    filters.forEach(filter => {
        document.getElementById(`filter-${filter}`).addEventListener('change', updateView);
    });
}

// Connect to WebSocket
function connectWebSocket() {
    const wsUrl = `ws://${window.location.hostname}:7890/ws`;
    const ws = new WebSocket(wsUrl);
    
    ws.onopen = () => {
        console.log('Connected to Canopy server');
        document.getElementById('status').textContent = 'Connected';
    };
    
    ws.onmessage = (event) => {
        const data = JSON.parse(event.data);
        if (data.type === 'graph_diff') {
            applyDiff(data.diff);
        } else if (data.type === 'full_graph') {
            currentGraph = data.graph;
            renderHierarchicalView();
        }
    };
    
    ws.onerror = (error) => {
        console.error('WebSocket error:', error);
        document.getElementById('status').textContent = 'Connection error';
    };
    
    ws.onclose = () => {
        console.log('Disconnected from Canopy server');
        document.getElementById('status').textContent = 'Disconnected';
        setTimeout(connectWebSocket, 3000);
    };
}

// Render hierarchical view (top-level modules only)
function renderHierarchicalView() {
    if (!currentGraph) return;
    
    const grid = document.getElementById('grid');
    grid.innerHTML = '';
    grid.className = '';
    
    // Get top-level modules (directories and root files)
    const topLevelNodes = currentGraph.nodes.filter(node => {
        const pathParts = node.file_path.split('/');
        return pathParts.length <= 2; // Root level only
    });
    
    // Group by type
    const directories = topLevelNodes.filter(n => n.kind === 'Directory');
    const files = topLevelNodes.filter(n => n.kind === 'File');
    const modules = [...directories, ...files];
    
    // Create module cards
    modules.forEach(module => {
        const card = createModuleCard(module);
        grid.appendChild(card);
    });
    
    // Show breadcrumb
    showBreadcrumb([]);
}

// Create a module card
function createModuleCard(module) {
    const card = document.createElement('div');
    card.className = 'module-card';
    card.onclick = () => selectModule(module);
    
    const icon = getModuleIcon(module.kind);
    const stats = getModuleStats(module);
    
    card.innerHTML = `
        <div class="module-header">
            <span class="module-icon">${icon}</span>
            <span class="module-name" title="${module.name}">${module.name}</span>
        </div>
        <div class="module-stats">
            <span class="stat">${icon} ${stats.count}</span>
            <span class="stat">📊 ${stats.lines} lines</span>
        </div>
        <div class="module-details">
            <div class="detail-row">
                <span class="detail-icon">📍</span>
                <span>${module.file_path}</span>
            </div>
            <div class="detail-row">
                <span class="detail-icon">🔤</span>
                <span>${module.language || 'Unknown'}</span>
            </div>
        </div>
    `;
    
    return card;
}

// Get icon for module type
function getModuleIcon(kind) {
    switch (kind) {
        case 'Directory': return '📁';
        case 'File': return '📄';
        case 'Function': return 'ƒ';
        case 'Class': return '○';
        case 'Struct': return '□';
        case 'Interface': return '△';
        default: return '📄';
    }
}

// Get module statistics
function getModuleStats(module) {
    // Count child nodes if available
    const children = currentGraph.nodes.filter(n => 
        n.file_path.startsWith(module.file_path + '/') ||
        (module.kind === 'Directory' && n.file_path.includes(module.name))
    );
    
    return {
        count: children.length,
        lines: module.line_end ? (module.line_end - (module.line_start || 0)) : 0
    };
}

// Select a module (drill down)
function selectModule(module) {
    selectedModule = module;
    
    // Remove previous selection
    document.querySelectorAll('.module-card').forEach(card => {
        card.classList.remove('selected');
    });
    
    // Add selection to clicked card
    event.currentTarget.classList.add('selected');
    
    // Drill down into the module
    drillDown(module);
}

// Drill down into a module
function drillDown(module) {
    currentPath.push(module.name);
    
    // Get children of this module
    const children = currentGraph.nodes.filter(node => {
        if (module.kind === 'Directory') {
            return node.file_path.startsWith(module.file_path + '/') &&
                   node.file_path !== module.file_path;
        } else {
            // For files, show contained items
            return node.file_path === module.file_path &&
                   node.kind !== 'File' && node.kind !== 'Directory';
        }
    });
    
    // Group children by level
    const groupedChildren = groupByLevel(children, module);
    
    // Render children
    renderChildren(groupedChildren);
    showBreadcrumb(currentPath);
}

// Group nodes by hierarchical level
function groupByLevel(nodes, parent) {
    const levels = {};
    
    nodes.forEach(node => {
        const relativePath = node.file_path.replace(parent.file_path, '').replace(/^\//, '');
        const level = relativePath.split('/').length - 1;
        
        if (!levels[level]) levels[level] = [];
        levels[level].push(node);
    });
    
    return levels;
}

// Render children in hierarchical view
function renderChildren(levels) {
    const grid = document.getElementById('grid');
    grid.innerHTML = '';
    
    Object.entries(levels).forEach(([level, nodes]) => {
        const levelDiv = document.createElement('div');
        levelDiv.className = 'hierarchy-level';
        
        nodes.forEach(node => {
            const card = createModuleCard(node);
            levelDiv.appendChild(card);
        });
        
        grid.appendChild(levelDiv);
    });
}

// Show breadcrumb navigation
function showBreadcrumb(path) {
    const breadcrumb = document.getElementById('breadcrumb');
    const currentPathSpan = document.getElementById('current-path');
    
    breadcrumb.style.display = 'flex';
    currentPathSpan.textContent = path.join(' / ');
}

// Navigate to root
function navigateToRoot() {
    currentPath = [];
    selectedModule = null;
    renderHierarchicalView();
}

// View controls
function viewHierarchical() {
    currentView = 'hierarchical';
    if (currentPath.length === 0) {
        renderHierarchicalView();
    } else {
        // Re-render current level in hierarchical mode
        drillDown(selectedModule);
    }
}

function viewFlat() {
    currentView = 'flat';
    renderFlatView();
}

// Render flat view (all nodes)
function renderFlatView() {
    if (!currentGraph) return;
    
    const grid = document.getElementById('grid');
    grid.innerHTML = '';
    
    // Apply filters
    const filteredNodes = filterNodes(currentGraph.nodes);
    
    // Create cards for all nodes
    filteredNodes.forEach(node => {
        const card = createModuleCard(node);
        grid.appendChild(card);
    });
}

// Filter nodes based on user selections
function filterNodes(nodes) {
    const filters = {
        directories: document.getElementById('filter-directories').checked,
        files: document.getElementById('filter-files').checked,
        functions: document.getElementById('filter-functions').checked,
        classes: document.getElementById('filter-classes').checked,
        ai: document.getElementById('filter-ai').checked
    };
    
    return nodes.filter(node => {
        switch (node.kind) {
            case 'Directory': return filters.directories;
            case 'File': return filters.files;
            case 'Function': return filters.functions;
            case 'Class':
            case 'Struct':
            case 'Interface': return filters.classes;
            default: return true;
        }
    });
}

// Handle search
function handleSearch(event) {
    const query = event.target.value.toLowerCase();
    const cards = document.querySelectorAll('.module-card');
    
    cards.forEach(card => {
        const name = card.querySelector('.module-name').textContent.toLowerCase();
        if (name.includes(query)) {
            card.style.display = 'block';
        } else {
            card.style.display = 'none';
        }
    });
}

// Update view when filters change
function updateView() {
    if (currentView === 'hierarchical') {
        if (currentPath.length === 0) {
            renderHierarchicalView();
        } else {
            drillDown(selectedModule);
        }
    } else {
        renderFlatView();
    }
}

// Apply graph diff (for real-time updates)
function applyDiff(diff) {
    if (!currentGraph) return;
    
    // Remove nodes
    diff.removed_nodes.forEach(node => {
        const index = currentGraph.nodes.findIndex(n => n.id === node.id);
        if (index !== -1) {
            currentGraph.nodes.splice(index, 1);
        }
    });
    
    // Add nodes
    currentGraph.nodes.push(...diff.added_nodes);
    
    // Update edges
    currentGraph.edges = currentGraph.edges.filter(edge => 
        !diff.removed_edges.some(re => re.id === edge.id)
    );
    currentGraph.edges.push(...diff.added_edges);
    
    // Re-render current view
    if (currentView === 'hierarchical') {
        if (currentPath.length === 0) {
            renderHierarchicalView();
        } else {
            drillDown(selectedModule);
        }
    } else {
        renderFlatView();
    }
}

// Initialize when DOM is ready
document.addEventListener('DOMContentLoaded', initGridView);