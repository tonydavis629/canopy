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
    
    // Set up zoom with mouse wheel
    const mainContainer = document.getElementById('main');
    mainContainer.addEventListener('wheel', handleZoom, { passive: false });
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

// Render hierarchical view (top-level modules and their concepts)
function renderHierarchicalView() {
    if (!currentGraph) return;
    
    const grid = document.getElementById('grid');
    grid.innerHTML = '';
    grid.className = '';
    
    // Get top-level modules
    const topLevelModules = currentGraph.nodes.filter(node => {
        const pathParts = node.file_path.split('/');
        return pathParts.length <= 2 && ['Directory', 'File'].includes(node.kind);
    });
    
    // For each top-level module, show its concepts
    topLevelModules.forEach(module => {
        const concepts = getModuleConcepts(module);
        
        if (concepts.length === 0) {
            // Just show the module card if no concepts
            const card = createModuleCard(module);
            grid.appendChild(card);
        } else {
            // Show module with its concepts
            const moduleSection = document.createElement('div');
            moduleSection.className = 'module-section';
            moduleSection.style.marginBottom = '24px';
            
            // Module header
            const header = document.createElement('div');
            header.className = 'module-header';
            header.innerHTML = `
                <span class="module-icon">${getModuleIcon(module.kind)}</span>
                <span class="module-name" style="font-size: 18px; font-weight: 500;">${module.name}</span>
                <span style="color: #9d9d9d; margin-left: 8px;">(${concepts.length} concepts)</span>
            `;
            moduleSection.appendChild(header);
            
            // Concepts grid
            const conceptsGrid = document.createElement('div');
            conceptsGrid.className = 'concepts-grid';
            conceptsGrid.style.display = 'grid';
            conceptsGrid.style.gridTemplateColumns = 'repeat(auto-fill, minmax(250px, 1fr))';
            conceptsGrid.style.gap = '12px';
            conceptsGrid.style.marginTop = '12px';
            conceptsGrid.style.marginLeft = '24px';
            
            concepts.forEach(concept => {
                const card = createConceptCardWithRelationships(concept, concepts);
                conceptsGrid.appendChild(card);
            });
            
            moduleSection.appendChild(conceptsGrid);
            grid.appendChild(moduleSection);
        }
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
    // Build proper path for the module
    let modulePath;
    if (module.kind === 'Directory') {
        modulePath = module.file_path;
    } else {
        // For files, use the directory path
        const parts = module.file_path.split('/');
        parts.pop(); // Remove filename
        modulePath = parts.join('/');
    }
    
    // Only add to path if it's a new level
    if (modulePath !== getCurrentPathString()) {
        currentPath.push({
            name: module.name,
            path: modulePath,
            kind: module.kind
        });
    }
    
    // Show back button when not at root
    const backButton = document.getElementById('back-button');
    if (currentPath.length > 0) {
        backButton.style.display = 'inline-block';
    }
    
    // Get ALL concepts (not just files) that belong to this module
    const moduleConcepts = getModuleConcepts(module);
    
    // Render concepts with their relationships
    renderConceptsWithRelationships(moduleConcepts, module);
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

// Show breadcrumb navigation with proper path
function showBreadcrumb(path) {
    const breadcrumb = document.getElementById('breadcrumb');
    const currentPathSpan = document.getElementById('current-path');
    
    if (path.length === 0) {
        breadcrumb.style.display = 'none';
        return;
    }
    
    breadcrumb.style.display = 'flex';
    
    // Build clean path
    const pathParts = path.map(p => p.name);
    currentPathSpan.textContent = pathParts.join(' / ');
}

// Render concepts with their relationships
function renderConceptsWithRelationships(concepts, parentModule) {
    const grid = document.getElementById('grid');
    grid.innerHTML = '';
    
    // Create a relationship graph container
    const graphContainer = document.createElement('div');
    graphContainer.className = 'concept-graph';
    graphContainer.style.display = 'flex';
    graphContainer.style.flexDirection = 'column';
    graphContainer.style.gap = '20px';
    
    // Group concepts by type
    const byType = {};
    concepts.forEach(concept => {
        if (!byType[concept.kind]) byType[concept.kind] = [];
        byType[concept.kind].push(concept);
    });
    
    // Render each type group
    Object.entries(byType).forEach(([kind, nodes]) => {
        const typeSection = document.createElement('div');
        typeSection.className = 'concept-section';
        
        // Section header
        const header = document.createElement('h3');
        header.textContent = `${kind}s`;
        header.style.marginBottom = '10px';
        header.style.color = '#9d9d9d';
        typeSection.appendChild(header);
        
        // Concept cards with relationships
        const cardsContainer = document.createElement('div');
        cardsContainer.className = 'concept-cards';
        cardsContainer.style.display = 'grid';
        cardsContainer.style.gridTemplateColumns = 'repeat(auto-fill, minmax(300px, 1fr))';
        cardsContainer.style.gap = '16px';
        
        nodes.forEach(node => {
            const card = createConceptCardWithRelationships(node, concepts);
            cardsContainer.appendChild(card);
        });
        
        typeSection.appendChild(cardsContainer);
        graphContainer.appendChild(typeSection);
    });
    
    grid.appendChild(graphContainer);
}

// Create a concept card with its relationships
function createConceptCardWithRelationships(concept, allConcepts) {
    const card = document.createElement('div');
    card.className = 'concept-card';
    card.style.background = '#2d2d30';
    card.style.border = '1px solid #3e3e42';
    card.style.borderRadius = '8px';
    card.style.padding = '16px';
    card.style.position = 'relative';
    
    // Get relationships for this concept
    const relationships = getConceptRelationships(concept, allConcepts);
    
    card.innerHTML = `
        <div class="concept-header">
            <span class="concept-icon">${getModuleIcon(concept.kind)}</span>
            <span class="concept-name">${concept.name}</span>
        </div>
        <div class="concept-details">
            <div class="detail-row">
                <span class="detail-icon">📍</span>
                <span>${concept.file_path}</span>
            </div>
            <div class="detail-row">
                <span class="detail-icon">📏</span>
                <span>Lines ${concept.line_start || 0}-${concept.line_end || 0}</span>
            </div>
        </div>
        ${relationships.length > 0 ? `
        <div class="concept-relationships" style="margin-top: 12px; padding-top: 12px; border-top: 1px solid #3e3e42;">
            <h4 style="margin: 0 0 8px 0; font-size: 14px; color: #9d9d9d;">Relationships</h4>
            ${relationships.map(rel => `
                <div class="relationship" style="display: flex; align-items: center; gap: 8px; margin: 4px 0; font-size: 13px;">
                    <span style="color: #0e639c;">${rel.relationship}</span>
                    <span>→</span>
                    <span>${rel.target.name}</span>
                </div>
            `).join('')}
        </div>
        ` : ''}
    `;
    
    return card;
}

// Get relationships for a concept
function getConceptRelationships(concept, allConcepts) {
    if (!currentGraph || !currentGraph.edges) return [];
    
    const relationships = [];
    
    currentGraph.edges.forEach(edge => {
        if (edge.source === concept.id) {
            const target = allConcepts.find(c => c.id === edge.target);
            if (target) {
                relationships.push({
                    relationship: getRelationshipName(edge.kind),
                    target: target
                });
            }
        } else if (edge.target === concept.id) {
            const source = allConcepts.find(c => c.id === edge.source);
            if (source) {
                relationships.push({
                    relationship: getRelationshipName(edge.kind) + " (from)",
                    target: source
                });
            }
        }
    });
    
    return relationships;
}

// Get human-readable relationship name
function getRelationshipName(kind) {
    const names = {
        'Contains': 'contains',
        'Calls': 'calls',
        'DependsOn': 'depends on',
        'Uses': 'uses',
        'Imports': 'imports',
        'Implements': 'implements',
        'Inherits': 'inherits from',
        'SemanticReference': 'references'
    };
    
    return names[kind] || kind;
}

// Navigate to root
function navigateToRoot() {
    currentPath = [];
    selectedModule = null;
    document.getElementById('back-button').style.display = 'none';
    renderHierarchicalView();
}

// Go back to parent level
function goBack() {
    if (currentPath.length === 0) return;
    
    currentPath.pop();
    
    // Hide back button if at root
    if (currentPath.length === 0) {
        document.getElementById('back-button').style.display = 'none';
        renderHierarchicalView();
    } else {
        // Go back to parent level
        const parentPath = currentPath[currentPath.length - 1];
        const parentModule = findModuleByPath(parentPath.path);
        if (parentModule) {
            drillDown(parentModule);
        } else {
            navigateToRoot();
        }
    }
}

// Get current path as string
function getCurrentPathString() {
    if (currentPath.length === 0) return '';
    return currentPath[currentPath.length - 1].path;
}

// Get ALL concepts (functions, classes, etc.) that belong to this module
function getModuleConcepts(module) {
    if (!currentGraph) return [];
    
    // Get the directory path for this module
    let moduleDir;
    if (module.kind === 'Directory') {
        moduleDir = module.file_path;
    } else {
        // For files, get the directory
        const parts = module.file_path.split('/');
        parts.pop();
        moduleDir = parts.join('/');
    }
    
    // Get all concepts in this directory
    const concepts = currentGraph.nodes.filter(node => {
        // Include the module itself
        if (node.id === module.id) return true;
        
        // Include concepts in the same directory
        const nodeDir = node.file_path.substring(0, node.file_path.lastIndexOf('/'));
        return nodeDir === moduleDir && 
               !['File', 'Directory'].includes(node.kind);
    });
    
    return concepts;
}

// Find module by path
function findModuleByPath(path) {
    if (!path) return null;
    
    // Find the module that matches this path
    return currentGraph.nodes.find(node => {
        if (node.kind === 'Directory') {
            return node.file_path === path || node.file_path.endsWith('/' + path);
        } else if (node.kind === 'File') {
            return node.file_path === path;
        }
        return false;
    });
}

// Show breadcrumb with proper path
function showBreadcrumb(path) {
    const breadcrumb = document.getElementById('breadcrumb');
    const currentPathSpan = document.getElementById('current-path');
    
    if (path.length === 0) {
        breadcrumb.style.display = 'none';
        return;
    }
    
    breadcrumb.style.display = 'flex';
    
    // Build clean path
    const pathParts = path.map(p => p.name);
    currentPathSpan.textContent = pathParts.join(' / ');
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

// Render flat view (all concepts)
function renderFlatView() {
    if (!currentGraph) return;
    
    const grid = document.getElementById('grid');
    grid.innerHTML = '';
    
    // Get all concepts (not just files/directories)
    const allConcepts = currentGraph.nodes.filter(node => 
        !['File', 'Directory'].includes(node.kind)
    );
    
    // Apply filters
    const filteredConcepts = filterNodes(allConcepts);
    
    // Group by file for organization
    const byFile = {};
    filteredConcepts.forEach(concept => {
        const file = concept.file_path;
        if (!byFile[file]) byFile[file] = [];
        byFile[file].push(concept);
    });
    
    // Render concepts grouped by file
    Object.entries(byFile).forEach(([file, concepts]) => {
        const fileSection = document.createElement('div');
        fileSection.className = 'file-section';
        fileSection.style.marginBottom = '20px';
        
        // File header
        const header = document.createElement('div');
        header.className = 'file-header';
        header.style.fontSize = '16px';
        header.style.fontWeight = '500';
        header.style.marginBottom = '10px';
        header.style.color = '#d4d4d4';
        header.textContent = file;
        fileSection.appendChild(header);
        
        // Concepts grid
        const conceptsGrid = document.createElement('div');
        conceptsGrid.className = 'concepts-grid';
        conceptsGrid.style.display = 'grid';
        conceptsGrid.style.gridTemplateColumns = 'repeat(auto-fill, minmax(250px, 1fr))';
        conceptsGrid.style.gap = '12px';
        
        concepts.forEach(concept => {
            const card = createConceptCardWithRelationships(concept, concepts);
            conceptsGrid.appendChild(card);
        });
        
        fileSection.appendChild(conceptsGrid);
        grid.appendChild(fileSection);
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

// Handle zoom with mouse wheel
function handleZoom(event) {
    event.preventDefault();
    
    const delta = event.deltaY;
    const zoomThreshold = 50; // Minimum scroll to trigger zoom
    
    if (Math.abs(delta) < zoomThreshold) return;
    
    if (delta > 0) {
        // Scroll down - zoom out (go higher in hierarchy)
        if (currentPath.length > 0) {
            goBack();
        }
    } else {
        // Scroll up - zoom in (if a module is selected)
        if (selectedModule) {
            drillDown(selectedModule);
        }
    }
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

// Export for use in other scripts
window.GridView = {
    renderHierarchicalView,
    renderFlatView,
    selectModule,
    navigateToRoot
};