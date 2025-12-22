// SQLite B-Tree Visualization

let currentBTree = null;
let currentZoom = null;
let simulation = null;

// Initialize visualization
function init() {
    renderDatabaseInfo();
    renderSchemaList();
    setupBTreeSelect();
    setupControls();
    setupResizeHandle();

    // Render first btree by default
    if (DATA.btrees.length > 0) {
        selectBTree(DATA.btrees[0].name);
    }
}

// Setup resizable detail panel
function setupResizeHandle() {
    const handle = document.getElementById('resize-handle');
    const app = document.getElementById('app');
    let isResizing = false;
    let startX = 0;
    let startWidth = 0;

    handle.addEventListener('mousedown', (e) => {
        isResizing = true;
        startX = e.clientX;
        const panel = document.getElementById('detail-panel');
        startWidth = panel.offsetWidth;
        handle.classList.add('dragging');
        document.body.style.cursor = 'ew-resize';
        document.body.style.userSelect = 'none';
        e.preventDefault();
    });

    document.addEventListener('mousemove', (e) => {
        if (!isResizing) return;

        const delta = startX - e.clientX;
        const newWidth = Math.min(Math.max(startWidth + delta, 250), 800);
        app.style.setProperty('--detail-panel-width', newWidth + 'px');
    });

    document.addEventListener('mouseup', () => {
        if (isResizing) {
            isResizing = false;
            handle.classList.remove('dragging');
            document.body.style.cursor = '';
            document.body.style.userSelect = '';
        }
    });
}

// Render database info in sidebar
function renderDatabaseInfo() {
    const container = document.getElementById('db-details');
    const info = DATA.database_info;

    container.innerHTML = `
        <p><span class="label">File:</span> <span class="value">${info.file_name}</span></p>
        <p><span class="label">Page Size:</span> <span class="value">${info.page_size} bytes</span></p>
        <p><span class="label">Pages:</span> <span class="value">${info.page_count}</span></p>
        <p><span class="label">Encoding:</span> <span class="value">${info.text_encoding}</span></p>
        <p><span class="label">SQLite:</span> <span class="value">${info.sqlite_version}</span></p>
    `;
}

// Render schema list in sidebar
function renderSchemaList() {
    const container = document.getElementById('schema-list');
    let html = '';

    // Tables
    DATA.schema.tables.forEach(table => {
        html += `
            <div class="schema-item" data-name="${table.name}" data-type="table">
                <div class="name">${table.name}</div>
                <div class="type">table</div>
                <div class="page">Page ${table.root_page}</div>
            </div>
        `;
    });

    // Indexes
    DATA.schema.indexes.forEach(index => {
        html += `
            <div class="schema-item" data-name="${index.name}" data-type="index">
                <div class="name">${index.name}</div>
                <div class="type">index on ${index.table_name}</div>
                <div class="page">Page ${index.root_page}</div>
            </div>
        `;
    });

    container.innerHTML = html;

    // Add click handlers
    container.querySelectorAll('.schema-item').forEach(item => {
        item.addEventListener('click', () => {
            const name = item.dataset.name;
            selectBTree(name);
        });
    });
}

// Setup B-tree select dropdown
function setupBTreeSelect() {
    const select = document.getElementById('btree-select');
    let html = '';

    DATA.btrees.forEach(btree => {
        html += `<option value="${btree.name}">${btree.name} (${btree.tree_type})</option>`;
    });

    select.innerHTML = html;
    select.addEventListener('change', (e) => selectBTree(e.target.value));
}

// Setup control buttons
function setupControls() {
    document.getElementById('zoom-in').addEventListener('click', () => {
        if (currentZoom) {
            const svg = d3.select('#tree-viz');
            svg.transition().call(currentZoom.scaleBy, 1.3);
        }
    });

    document.getElementById('zoom-out').addEventListener('click', () => {
        if (currentZoom) {
            const svg = d3.select('#tree-viz');
            svg.transition().call(currentZoom.scaleBy, 0.7);
        }
    });

    document.getElementById('zoom-reset').addEventListener('click', () => {
        if (currentZoom) {
            const svg = d3.select('#tree-viz');
            svg.transition().call(currentZoom.transform, d3.zoomIdentity);
        }
    });

    document.getElementById('view-mode').addEventListener('change', (e) => {
        if (currentBTree) {
            if (e.target.value === 'tree') {
                renderTreeView(currentBTree);
            } else {
                renderForceView(currentBTree);
            }
        }
    });
}

// Select and render a B-tree
function selectBTree(name) {
    const btree = DATA.btrees.find(b => b.name === name);
    if (!btree) return;

    currentBTree = btree;

    // Update sidebar selection
    document.querySelectorAll('.schema-item').forEach(item => {
        item.classList.toggle('active', item.dataset.name === name);
    });

    // Update dropdown
    document.getElementById('btree-select').value = name;

    // Render based on current view mode
    const viewMode = document.getElementById('view-mode').value;
    if (viewMode === 'tree') {
        renderTreeView(btree);
    } else {
        renderForceView(btree);
    }

    // Clear page details
    clearPageDetails();
}

// Render B-tree as hierarchical tree
function renderTreeView(btree) {
    const svg = d3.select('#tree-viz');
    svg.selectAll('*').remove();

    const container = document.getElementById('viz-container');
    const width = container.clientWidth;
    const height = container.clientHeight;

    svg.attr('viewBox', [0, 0, width, height]);

    // Build hierarchy
    const root = buildHierarchy(btree);
    if (!root) return;

    // Create tree layout
    const treeLayout = d3.tree()
        .size([width - 100, height - 100])
        .separation((a, b) => (a.parent === b.parent ? 1 : 1.5));

    const treeData = treeLayout(d3.hierarchy(root));

    // Create container group for zoom
    const g = svg.append('g')
        .attr('transform', 'translate(50, 50)');

    // Setup zoom
    currentZoom = d3.zoom()
        .scaleExtent([0.1, 4])
        .on('zoom', (event) => {
            g.attr('transform', event.transform);
        });

    svg.call(currentZoom);

    // Render links
    g.selectAll('.link')
        .data(treeData.links())
        .join('path')
        .attr('class', 'link')
        .attr('d', d3.linkVertical()
            .x(d => d.x)
            .y(d => d.y));

    // Render nodes
    const nodes = g.selectAll('.node')
        .data(treeData.descendants())
        .join('g')
        .attr('class', 'node')
        .attr('transform', d => `translate(${d.x},${d.y})`)
        .on('click', (event, d) => showPageDetails(d.data.page_number))
        .on('mouseover', showTooltip)
        .on('mouseout', hideTooltip);

    nodes.append('circle')
        .attr('r', d => Math.sqrt(d.data.cell_count || 1) * 4 + 8)
        .attr('class', d => getPageClass(d.data.page_type));

    nodes.append('text')
        .attr('dy', 4)
        .attr('text-anchor', 'middle')
        .text(d => d.data.page_number);
}

// Render B-tree as force-directed graph
function renderForceView(btree) {
    const svg = d3.select('#tree-viz');
    svg.selectAll('*').remove();

    const container = document.getElementById('viz-container');
    const width = container.clientWidth;
    const height = container.clientHeight;

    svg.attr('viewBox', [0, 0, width, height]);

    // Prepare nodes and links
    const nodes = btree.nodes.map(n => ({
        id: n.id,
        page_number: n.page_number,
        page_type: n.page_type,
        cell_count: n.cell_count,
        depth: n.depth
    }));

    const links = btree.links
        .filter(l => l.link_type === 'child')
        .map(l => ({
            source: l.source,
            target: l.target
        }));

    // Create container group for zoom
    const g = svg.append('g');

    // Setup zoom
    currentZoom = d3.zoom()
        .scaleExtent([0.1, 4])
        .on('zoom', (event) => {
            g.attr('transform', event.transform);
        });

    svg.call(currentZoom);

    // Create simulation
    simulation = d3.forceSimulation(nodes)
        .force('link', d3.forceLink(links).id(d => d.id).distance(80))
        .force('charge', d3.forceManyBody().strength(-200))
        .force('center', d3.forceCenter(width / 2, height / 2))
        .force('y', d3.forceY().y(d => 100 + d.depth * 100).strength(0.5));

    // Render links
    const link = g.selectAll('.link')
        .data(links)
        .join('line')
        .attr('class', 'link')
        .attr('stroke', '#ccc')
        .attr('stroke-width', 1.5);

    // Render nodes
    const node = g.selectAll('.node')
        .data(nodes)
        .join('g')
        .attr('class', 'node')
        .on('click', (event, d) => showPageDetails(d.page_number))
        .on('mouseover', showTooltip)
        .on('mouseout', hideTooltip)
        .call(d3.drag()
            .on('start', dragstarted)
            .on('drag', dragged)
            .on('end', dragended));

    node.append('circle')
        .attr('r', d => Math.sqrt(d.cell_count || 1) * 4 + 8)
        .attr('class', d => getPageClass(d.page_type));

    node.append('text')
        .attr('dy', 4)
        .attr('text-anchor', 'middle')
        .text(d => d.page_number);

    simulation.on('tick', () => {
        link
            .attr('x1', d => d.source.x)
            .attr('y1', d => d.source.y)
            .attr('x2', d => d.target.x)
            .attr('y2', d => d.target.y);

        node.attr('transform', d => `translate(${d.x},${d.y})`);
    });

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

// Build hierarchy from btree nodes
function buildHierarchy(btree) {
    if (btree.nodes.length === 0) return null;

    const nodeMap = new Map();
    btree.nodes.forEach(n => nodeMap.set(n.page_number, { ...n, children: [] }));

    let root = null;

    btree.nodes.forEach(node => {
        const n = nodeMap.get(node.page_number);
        node.children.forEach(childPage => {
            const child = nodeMap.get(childPage);
            if (child) {
                n.children.push(child);
            }
        });

        if (node.page_number === btree.root_page) {
            root = n;
        }
    });

    return root;
}

// Get CSS class for page type
function getPageClass(pageType) {
    const typeMap = {
        'InteriorTable': 'page-interior-table',
        'LeafTable': 'page-leaf-table',
        'InteriorIndex': 'page-interior-index',
        'LeafIndex': 'page-leaf-index',
        'Overflow': 'page-overflow'
    };
    return typeMap[pageType] || 'page-leaf-table';
}

// Current page for detail view
let currentDetailPage = null;
let currentCellLookup = null;

// Show page details
function showPageDetails(pageNum) {
    const page = DATA.pages.find(p => p.page_number === pageNum);
    if (!page) {
        document.getElementById('page-info').innerHTML = `<p>Page ${pageNum} details not available</p>`;
        return;
    }

    // Update info panel
    document.getElementById('page-info').innerHTML = `
        <div class="info-row">
            <span class="info-label">Page Number</span>
            <span class="info-value">${page.page_number}</span>
        </div>
        <div class="info-row">
            <span class="info-label">Type</span>
            <span class="info-value">${page.page_type}</span>
        </div>
        <div class="info-row">
            <span class="info-label">Cells</span>
            <span class="info-value">${page.cell_count}</span>
        </div>
        <div class="info-row">
            <span class="info-label">Free Space</span>
            <span class="info-value">${page.free_space} bytes</span>
        </div>
        <button class="view-page-btn" onclick="openPageDetailView(${page.page_number})">View Page Structure</button>
    `;

    // Render page structure
    renderPageStructure(page);

    // Render cells list
    renderCellsList(page);
}

// Render page internal structure
function renderPageStructure(page) {
    const svg = d3.select('#page-viz');
    svg.selectAll('*').remove();

    const pageSize = DATA.database_info.page_size;
    const width = 260;
    const height = 180;
    const margin = 10;

    svg.attr('viewBox', [0, 0, width, height]);

    const barWidth = width - 2 * margin;
    const barHeight = height - 2 * margin;
    const scale = barHeight / pageSize;

    const g = svg.append('g').attr('transform', `translate(${margin}, ${margin})`);

    // Page background
    g.append('rect')
        .attr('width', barWidth)
        .attr('height', barHeight)
        .attr('fill', '#f0f0f0')
        .attr('stroke', '#ccc');

    // Header area (estimate)
    const headerSize = page.page_number === 1 ? 108 : 8;
    g.append('rect')
        .attr('width', barWidth)
        .attr('height', headerSize * scale)
        .attr('fill', '#bdc3c7');

    // Cell pointer array (estimate)
    const cellPointerSize = page.cell_count * 2;
    g.append('rect')
        .attr('y', headerSize * scale)
        .attr('width', barWidth)
        .attr('height', cellPointerSize * scale)
        .attr('fill', '#95a5a6');

    // Cells
    page.cells.forEach((cell, i) => {
        const y = cell.offset * scale;
        const h = Math.max(cell.size * scale, 2);

        g.append('rect')
            .attr('y', y)
            .attr('width', barWidth)
            .attr('height', h)
            .attr('fill', cell.has_overflow ? '#e74c3c' : '#3498db')
            .attr('stroke', '#fff')
            .attr('stroke-width', 0.5);
    });

    // Legend
    const legend = svg.append('g').attr('transform', `translate(${width - 80}, 5)`);
    const items = [
        { color: '#bdc3c7', label: 'Header' },
        { color: '#95a5a6', label: 'Pointers' },
        { color: '#3498db', label: 'Cells' },
        { color: '#e74c3c', label: 'Overflow' }
    ];

    items.forEach((item, i) => {
        legend.append('rect')
            .attr('x', 0)
            .attr('y', i * 12)
            .attr('width', 10)
            .attr('height', 10)
            .attr('fill', item.color);
        legend.append('text')
            .attr('x', 14)
            .attr('y', i * 12 + 9)
            .attr('font-size', 8)
            .text(item.label);
    });
}

// Render cells list with expandable content
function renderCellsList(page) {
    const container = document.getElementById('cells');
    let html = '';

    page.cells.forEach(cell => {
        html += `
            <div class="cell-item ${cell.has_overflow ? 'has-overflow' : ''}" onclick="toggleCellExpand(this)">
                <div class="cell-header">
                    <span class="cell-index">#${cell.index}</span>
                    <span class="cell-type">${cell.cell_type}</span>
                </div>
                <div class="cell-preview">${escapeHtml(cell.preview)}</div>
                <div class="cell-meta">
                    ${cell.rowid !== null ? `<span>Rowid: ${cell.rowid}</span>` : ''}
                    <span>Offset: ${cell.offset}</span>
                    <span>Size: ${cell.size}B</span>
                    ${cell.left_child !== null ? `<span>Child: P${cell.left_child}</span>` : ''}
                </div>
                ${cell.has_overflow ? `<div style="color: #e74c3c; font-size: 11px; margin-top: 3px">Overflow: Page ${cell.overflow_page}</div>` : ''}
                <div class="expand-hint">Click to expand</div>
                <div class="cell-full-content">
                    <pre>${escapeHtml(cell.full_content || cell.preview)}</pre>
                </div>
            </div>
        `;
    });

    container.innerHTML = html || '<p class="placeholder">No cells</p>';
}

// Toggle cell expansion
function toggleCellExpand(element) {
    element.classList.toggle('expanded');
}

// Clear page details panel
function clearPageDetails() {
    document.getElementById('page-info').innerHTML = '<p class="placeholder">Click a page to view details</p>';
    d3.select('#page-viz').selectAll('*').remove();
    document.getElementById('cells').innerHTML = '';
}

// Tooltip functions
function showTooltip(event, d) {
    const tooltip = d3.select('body').append('div')
        .attr('class', 'tooltip')
        .style('left', (event.pageX + 10) + 'px')
        .style('top', (event.pageY - 10) + 'px');

    tooltip.html(`
        <div class="tip-title">Page ${d.data ? d.data.page_number : d.page_number}</div>
        <div class="tip-row"><span class="tip-label">Type:</span> ${d.data ? d.data.page_type : d.page_type}</div>
        <div class="tip-row"><span class="tip-label">Cells:</span> ${d.data ? d.data.cell_count : d.cell_count}</div>
    `);
}

function hideTooltip() {
    d3.selectAll('.tooltip').remove();
}

// Utility
function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// ===============================
// Page Detail View Functions
// ===============================

// Open the detailed page view
function openPageDetailView(pageNum) {
    const page = DATA.pages.find(p => p.page_number === pageNum);
    if (!page) return;

    currentDetailPage = page;

    // Update title
    document.getElementById('page-detail-title').textContent =
        `Page ${page.page_number} | ${page.page_type}`;

    // Render the byte grid
    renderPageGrid(page);

    // Render cells in sidebar
    renderPageDetailCells(page);

    // Show the view
    document.getElementById('page-detail-view').classList.add('active');
    document.body.style.overflow = 'hidden';
}

// Close the detailed page view
function closePageDetailView() {
    document.getElementById('page-detail-view').classList.remove('active');
    document.body.style.overflow = '';
    currentDetailPage = null;
    currentCellLookup = null;
}

// Render the page as a byte grid
function renderPageGrid(page) {
    const container = document.getElementById('page-grid');
    const pageSize = DATA.database_info.page_size;
    const bytesPerRow = 64;
    const rows = Math.ceil(pageSize / bytesPerRow);

    // Calculate regions
    const isPage1 = page.page_number === 1;
    const dbHeaderEnd = isPage1 ? 100 : 0;
    const pageHeaderStart = isPage1 ? 100 : 0;
    const isInterior = page.page_type.includes('Interior');
    const pageHeaderSize = isInterior ? 12 : 8;
    const pageHeaderEnd = pageHeaderStart + pageHeaderSize;
    const cellPointerEnd = pageHeaderEnd + (page.cell_count * 2);
    const cellContentStart = page.cell_content_start || pageSize;

    // Build cell regions map with pointer index
    const cellRegions = [];
    page.cells.forEach((cell, idx) => {
        cellRegions.push({
            ptrIndex: idx,  // Index in pointer array (logical order)
            start: cell.offset,
            end: cell.offset + cell.size,
            hasOverflow: cell.has_overflow
        });
    });

    // Sort by offset to get physical order
    const cellsByOffset = [...cellRegions].sort((a, b) => a.start - b.start);
    // Assign physical order index
    cellsByOffset.forEach((cell, physIdx) => {
        cell.physIndex = physIdx;
    });
    // Create lookup by pointer index
    const cellLookup = new Map();
    cellRegions.forEach(cell => {
        cellLookup.set(cell.ptrIndex, cell);
    });
    // Store globally for use by sidebar cell clicks
    currentCellLookup = cellLookup;

    let html = '';

    for (let row = 0; row < rows; row++) {
        html += '<div class="page-grid-row">';
        for (let col = 0; col < bytesPerRow; col++) {
            const byteOffset = row * bytesPerRow + col;
            if (byteOffset >= pageSize) break;

            let className = 'page-byte';
            let dataAttrs = `data-offset="${byteOffset}"`;

            // Determine byte type
            if (isPage1 && byteOffset < 100) {
                className += ' header';
                dataAttrs += ' data-type="db-header"';
            } else if (byteOffset >= pageHeaderStart && byteOffset < pageHeaderEnd) {
                className += ' header';
                dataAttrs += ' data-type="page-header"';
            } else if (byteOffset >= pageHeaderEnd && byteOffset < cellPointerEnd) {
                className += ' cell-pointer';
                const ptrIndex = Math.floor((byteOffset - pageHeaderEnd) / 2);
                dataAttrs += ` data-type="cell-pointer" data-ptr-index="${ptrIndex}"`;
            } else {
                // Check if in a cell
                let inCell = null;
                for (const region of cellRegions) {
                    if (byteOffset >= region.start && byteOffset < region.end) {
                        inCell = region;
                        break;
                    }
                }

                if (inCell) {
                    className += ' cell-content';
                    if (inCell.hasOverflow) className += ' overflow';
                    dataAttrs += ` data-type="cell" data-cell-index="${inCell.ptrIndex}" data-phys-index="${inCell.physIndex}"`;
                    // Mark first byte of cell for label placement
                    if (byteOffset === inCell.start) {
                        dataAttrs += ' data-cell-start="true"';
                    }
                } else {
                    className += ' free';
                    dataAttrs += ' data-type="free"';
                }
            }

            html += `<div class="${className}" ${dataAttrs}></div>`;
        }
        html += '</div>';
    }

    container.innerHTML = html;

    // Add cell order labels at the start of each cell
    container.querySelectorAll('.page-byte[data-cell-start="true"]').forEach(el => {
        const ptrIndex = parseInt(el.dataset.cellIndex);
        const physIndex = parseInt(el.dataset.physIndex);
        const label = document.createElement('div');
        label.className = 'cell-order-label';
        label.innerHTML = `<span class="ptr-order">P${ptrIndex}</span> <span class="phys-order">→${physIndex}</span>`;
        label.title = `Pointer order: #${ptrIndex}, Physical order: #${physIndex}`;
        el.style.position = 'relative';
        el.appendChild(label);
    });

    // Add click handlers for cell pointers
    container.querySelectorAll('.cell-pointer').forEach(el => {
        el.addEventListener('click', (e) => {
            const ptrIndex = parseInt(el.dataset.ptrIndex);
            selectCellPointer(ptrIndex, page, cellLookup);
        });
    });

    // Add click handlers for cells
    container.querySelectorAll('.cell-content').forEach(el => {
        el.addEventListener('click', (e) => {
            const cellIndex = parseInt(el.dataset.cellIndex);
            selectCell(cellIndex, page, cellLookup);
        });
    });
}

// Select a cell pointer and highlight the corresponding cell
function selectCellPointer(ptrIndex, page, cellLookup) {
    // Clear previous selection
    document.querySelectorAll('.page-byte.selected, .page-byte.highlighted').forEach(el => {
        el.classList.remove('selected', 'highlighted');
    });

    // Highlight the pointer bytes
    const isInterior = page.page_type.includes('Interior');
    const pageHeaderSize = isInterior ? 12 : 8;
    const isPage1 = page.page_number === 1;
    const pageHeaderStart = isPage1 ? 100 : 0;
    const ptrStart = pageHeaderStart + pageHeaderSize + (ptrIndex * 2);

    document.querySelectorAll(`.page-byte[data-offset="${ptrStart}"], .page-byte[data-offset="${ptrStart + 1}"]`).forEach(el => {
        el.classList.add('selected');
    });

    // Get the cell this pointer points to
    const cell = page.cells[ptrIndex];
    const cellInfo = cellLookup ? cellLookup.get(ptrIndex) : null;
    if (cell) {
        // Highlight the cell content
        for (let i = cell.offset; i < cell.offset + cell.size; i++) {
            const el = document.querySelector(`.page-byte[data-offset="${i}"]`);
            if (el) el.classList.add('highlighted');
        }

        // Update selection info
        showPointerInfo(ptrIndex, cell, page, cellInfo);
    }
}

// Select a cell and show its details
function selectCell(cellIndex, page, cellLookup) {
    // Clear previous selection
    document.querySelectorAll('.page-byte.selected, .page-byte.highlighted').forEach(el => {
        el.classList.remove('selected', 'highlighted');
    });

    const cell = page.cells[cellIndex];
    if (!cell) return;

    const cellInfo = cellLookup ? cellLookup.get(cellIndex) : null;

    // Highlight the cell
    for (let i = cell.offset; i < cell.offset + cell.size; i++) {
        const el = document.querySelector(`.page-byte[data-offset="${i}"]`);
        if (el) el.classList.add('highlighted');
    }

    // Also highlight the corresponding pointer
    const isInterior = page.page_type.includes('Interior');
    const pageHeaderSize = isInterior ? 12 : 8;
    const isPage1 = page.page_number === 1;
    const pageHeaderStart = isPage1 ? 100 : 0;
    const ptrStart = pageHeaderStart + pageHeaderSize + (cellIndex * 2);

    document.querySelectorAll(`.page-byte[data-offset="${ptrStart}"], .page-byte[data-offset="${ptrStart + 1}"]`).forEach(el => {
        el.classList.add('selected');
    });

    showCellInfo(cellIndex, cell, page, cellInfo);
}

// Show pointer info in sidebar
function showPointerInfo(ptrIndex, cell, page, cellInfo) {
    const isInterior = page.page_type.includes('Interior');
    const pageHeaderSize = isInterior ? 12 : 8;
    const isPage1 = page.page_number === 1;
    const pageHeaderStart = isPage1 ? 100 : 0;
    const ptrOffset = pageHeaderStart + pageHeaderSize + (ptrIndex * 2);
    const fileOffset = (page.page_number - 1) * DATA.database_info.page_size + ptrOffset;
    const cellFileOffset = (page.page_number - 1) * DATA.database_info.page_size + cell.offset;
    const physIndex = cellInfo ? cellInfo.physIndex : '?';

    document.getElementById('selection-info').innerHTML = `
        <div class="pointer-info">
            <h4>Cell Pointer #${ptrIndex}</h4>
            <div class="meta">File Offset: ${fileOffset} · Page Offset: ${ptrOffset} · Length: 2</div>
            <p style="font-size: 12px; margin-bottom: 10px;">Points to cell content at offset ${cell.offset}</p>
            <table class="detail-table">
                <tr><th>Field</th><th>Value</th></tr>
                <tr><td>Pointer Index (logical)</td><td>${ptrIndex}</td></tr>
                <tr><td>Physical Order</td><td>${physIndex}</td></tr>
                <tr><td>Cell Offset</td><td>${cell.offset}</td></tr>
                <tr><td>Cell File Offset</td><td>${cellFileOffset}</td></tr>
                <tr><td>Cell Size</td><td>${cell.size} bytes</td></tr>
                ${cell.rowid !== null ? `<tr><td>Rowid</td><td>${cell.rowid}</td></tr>` : ''}
            </table>
            <h4 style="margin-top: 15px;">Full Content</h4>
            <pre style="background: #f5f5f5; padding: 10px; border-radius: 4px; font-size: 11px; overflow-x: auto; white-space: pre-wrap; word-break: break-all; max-height: 200px; overflow-y: auto;">${escapeHtml(cell.full_content || cell.preview)}</pre>
        </div>
    `;
}

// Show cell info in sidebar
function showCellInfo(cellIndex, cell, page, cellInfo) {
    const fileOffset = (page.page_number - 1) * DATA.database_info.page_size + cell.offset;
    const physIndex = cellInfo ? cellInfo.physIndex : '?';

    document.getElementById('selection-info').innerHTML = `
        <div class="cell-detail-info">
            <h4>Cell #${cellIndex} (${cell.cell_type})</h4>
            <div class="meta">File Offset: ${fileOffset} · Page Offset: ${cell.offset} · Size: ${cell.size}</div>
            <table class="detail-table">
                <tr><th>Field</th><th>Value</th></tr>
                <tr><td>Pointer Index (logical)</td><td>${cellIndex}</td></tr>
                <tr><td>Physical Order</td><td>${physIndex}</td></tr>
                <tr><td>Offset</td><td>${cell.offset}</td></tr>
                <tr><td>Size</td><td>${cell.size} bytes</td></tr>
                ${cell.rowid !== null ? `<tr><td>Rowid</td><td>${cell.rowid}</td></tr>` : ''}
                ${cell.payload_size ? `<tr><td>Payload Size</td><td>${cell.payload_size} bytes</td></tr>` : ''}
                ${cell.left_child !== null ? `<tr><td>Left Child</td><td>Page ${cell.left_child}</td></tr>` : ''}
                ${cell.has_overflow ? `<tr><td>Overflow</td><td>Page ${cell.overflow_page}</td></tr>` : ''}
            </table>
            <h4 style="margin-top: 15px;">Full Content</h4>
            <pre style="background: #f5f5f5; padding: 10px; border-radius: 4px; font-size: 11px; overflow-x: auto; white-space: pre-wrap; word-break: break-all; max-height: 300px; overflow-y: auto;">${escapeHtml(cell.full_content || cell.preview)}</pre>
        </div>
    `;
}

// Render cells list in page detail sidebar
function renderPageDetailCells(page) {
    const container = document.getElementById('page-detail-cells');
    let html = '';

    page.cells.forEach((cell, idx) => {
        html += `
            <div class="cell-item ${cell.has_overflow ? 'has-overflow' : ''}" onclick="selectCell(${idx}, currentDetailPage, currentCellLookup)" style="cursor: pointer;">
                <div class="cell-header">
                    <span class="cell-index">#${idx}</span>
                    <span class="cell-type">${cell.cell_type}</span>
                </div>
                <div class="cell-preview">${escapeHtml(cell.preview)}</div>
                <div class="cell-meta">
                    ${cell.rowid !== null ? `<span>Rowid: ${cell.rowid}</span>` : ''}
                    <span>Size: ${cell.size}B</span>
                    ${cell.left_child !== null ? `<span>Child: P${cell.left_child}</span>` : ''}
                </div>
                ${cell.has_overflow ? `<div style="color: #e74c3c; font-size: 11px; margin-top: 3px">Overflow: Page ${cell.overflow_page}</div>` : ''}
            </div>
        `;
    });

    container.innerHTML = html || '<p class="placeholder">No cells</p>';
}

// Handle escape key to close page detail view
document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape' && currentDetailPage) {
        closePageDetailView();
    }
});

// Start
document.addEventListener('DOMContentLoaded', init);
