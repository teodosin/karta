<script lang="ts">
    import { onMount } from "svelte";
    import { browser } from "$app/environment";

    // State variables
    let canvasContainer: HTMLElement;
    let canvas: HTMLElement;
    let edgesLayer: SVGElement;

    // Instead of using panzoom library's event handling, we'll implement our own
    let scale = 1;
    let posX = 0;
    let posY = 0;
    let isPanning = false;
    let startX = 0;
    let startY = 0;

    // Mode tracking
    let currentMode = $state("move"); // 'move' or 'connect'

    // Node tracking
    let nodes = $state<
        Array<{
            element: HTMLElement;
            id: string;
            x: number;
            y: number;
        }>
    >([]);

    // Node map for faster lookups
    let nodeMap = $state<
        Record<
            string,
            {
                element: HTMLElement;
                x: number;
                y: number;
            }
        >
    >({});

    let edges = $state<
        Array<{
            id: string;
            source: string;
            target: string;
            element: SVGPathElement;
        }>
    >([]);

    // Edge map for faster lookups
    let edgesByNode = $state<Record<string, string[]>>({});

    // Interaction state
    let isDraggingNode = $state(false);
    let currentNode = $state<HTMLElement | null>(null);
    let sourceNode = $state<HTMLElement | null>(null);
    let nodeOffsetX = $state(0);
    let nodeOffsetY = $state(0);

    // Throttling state
    let edgeUpdatePending = false;

    onMount(async () => {
        if (browser) {
            // Initialize nodes
            createInitialNodes();

            // Set initial transform
            updateTransform();
        }
    });

    // Update transform function to replace panzoom
    function updateTransform() {
        canvas.style.transform = `translate(${posX}px, ${posY}px) scale(${scale})`;
    }

    // Create initial nodes in a grid
    function createInitialNodes() {
        const gridSize = 3;
        const spacing = 150;
        const startX = (canvas.clientWidth - gridSize * spacing) / 2;
        const startY = (canvas.clientHeight - gridSize * spacing) / 2;

        // Create a document fragment for batch insertion
        const fragment = document.createDocumentFragment();

        for (let i = 0; i < gridSize; i++) {
            for (let j = 0; j < gridSize; j++) {
                createNode(
                    startX + j * spacing,
                    startY + i * spacing,
                    `${i * gridSize + j + 1}`,
                    fragment,
                );
            }
        }

        // Add all nodes at once
        canvas.appendChild(fragment);
    }

    // Stress test function to create a large number of nodes and connections
    function createStressTest() {
        // Reset any existing nodes and edges
        nodes.forEach((node) => {
            if (node.element.parentNode) {
                node.element.parentNode.removeChild(node.element);
            }
        });

        edges.forEach((edge) => {
            if (edge.element.parentNode) {
                edge.element.parentNode.removeChild(edge.element);
            }
        });

        nodes = [];
        edges = [];
        nodeMap = {};
        edgesByNode = {};

        // Clear any temporary lines
        const tempLine = document.getElementById("temp-line");
        if (tempLine && tempLine.parentNode) {
            tempLine.parentNode.removeChild(tempLine);
        }

        // Create nodes in a grid
        const gridSize = 30;
        const connectionsPerNode = 2;
        const spacing = 120;
        const startX = (canvas.clientWidth - gridSize * spacing) / 2;
        const startY = (canvas.clientHeight - gridSize * spacing) / 2;

        // Create a document fragment for batch insertion
        const fragment: DocumentFragment | null =
            document.createDocumentFragment();

        // First create all nodes
        for (let i = 0; i < gridSize; i++) {
            for (let j = 0; j < gridSize; j++) {
                const x = startX + j * spacing;
                const y = startY + i * spacing;
                const label = `${i * gridSize + j + 1}`;
                createNode(x, y, label, fragment);
            }
        }

        // Add all nodes at once
        canvas.appendChild(fragment);

        // Prepare SVG fragment for edges
        const svgFragment: DocumentFragment | null =
            document.createDocumentFragment();

        // Then create connections - random connections
        for (let i = 0; i < nodes.length; i++) {
            const sourceId = nodes[i].id;

            // Create a set of unique random target indices
            const targetIndices = new Set<number>();
            while (targetIndices.size < connectionsPerNode) {
                const randomIndex = Math.floor(Math.random() * nodes.length);
                if (randomIndex !== i) {
                    // Don't connect to self
                    targetIndices.add(randomIndex);
                }
            }

            // Create edges to all target nodes
            for (const targetIndex of targetIndices) {
                const targetId = nodes[targetIndex].id;
                createEdge(sourceId, targetId, svgFragment, false);
            }
        }

        // Add all edges at once
        edgesLayer.appendChild(svgFragment);

        // Update all edges at once
        updateAllEdges();

        // Reset view to see the whole grid
        scale = 0.2;
        posX = 0;
        posY = 0;
        updateTransform();
    }

    // Create a node
    function createNode(
        x: number,
        y: number,
        label: string,
        fragment: DocumentFragment | null = null,
    ) {
        const node = document.createElement("div");
        // Apply Tailwind classes directly
        node.className =
            "node w-[100px] h-[100px] bg-indigo-600 text-white flex items-center justify-center font-bold rounded absolute select-none cursor-move shadow-md transition-shadow";
        node.textContent = label;
        node.style.left = `${x}px`;
        node.style.top = `${y}px`;
        node.dataset.id = `node-${nodes.length}`;

        // Add to fragment if provided, otherwise directly to canvas
        if (fragment) {
            fragment.appendChild(node);
        } else {
            canvas.appendChild(node);
        }

        const nodeId = node.dataset.id;

        // Add to nodes array
        nodes = [
            ...nodes,
            {
                element: node,
                id: nodeId,
                x: x,
                y: y,
            },
        ];

        // Add to node map for faster lookups
        nodeMap[nodeId] = {
            element: node,
            x: x,
            y: y,
        };

        // Initialize edges array for this node
        edgesByNode[nodeId] = [];

        // Add hover and selected state event listeners
        node.addEventListener("mouseenter", () => {
            node.classList.add("shadow-lg", "z-10");
        });

        node.addEventListener("mouseleave", () => {
            node.classList.remove("shadow-lg", "z-10");
        });

        // Node event listeners for dragging
        node.addEventListener("mousedown", (e) => handleNodeMouseDown(e, node));

        return node;
    }

    // Handle node mousedown
    function handleNodeMouseDown(e: MouseEvent, node: HTMLElement) {
        e.stopPropagation();

        if (e.button === 0) {
            // Left click
            e.preventDefault();

            if (currentMode === "move") {
                isDraggingNode = true;
                currentNode = node;
                node.classList.add("ring-2", "ring-yellow-400");

                // Calculate offset from node's top-left corner in canvas coordinates
                const rect = node.getBoundingClientRect();
                const canvasRect = canvasContainer.getBoundingClientRect();

                // Convert screen coordinates to canvas coordinates
                nodeOffsetX = e.clientX - rect.left;
                nodeOffsetY = e.clientY - rect.top;
            } else if (currentMode === "connect") {
                sourceNode = node;

                // Create temporary line for visual feedback
                const tempLine = document.createElementNS(
                    "http://www.w3.org/2000/svg",
                    "line",
                );
                tempLine.id = "temp-line";
                tempLine.setAttribute("stroke", "#2196F3");
                tempLine.setAttribute("stroke-width", "2");
                tempLine.setAttribute("stroke-dasharray", "5,5");

                // Get starting position
                const sourceNodePos = getNodeCenter(node);
                tempLine.setAttribute("x1", sourceNodePos.x.toString());
                tempLine.setAttribute("y1", sourceNodePos.y.toString());
                tempLine.setAttribute("x2", sourceNodePos.x.toString());
                tempLine.setAttribute("y2", sourceNodePos.y.toString());

                edgesLayer.appendChild(tempLine);
            }
        }
    }

    // Get center point of a node
    function getNodeCenter(nodeElement: HTMLElement) {
        const nodeId = nodeElement.dataset.id;
        if (!nodeId) return { x: 0, y: 0 };
        const nodeData = nodeMap[nodeId];
        if (!nodeData) return { x: 0, y: 0 };

        return {
            x: nodeData.x + 50,
            y: nodeData.y + 50,
        };
    }

    // Update node position
    function updateNodePosition(node: HTMLElement, x: number, y: number) {
        node.style.left = `${x}px`;
        node.style.top = `${y}px`;

        // Update stored position
        const nodeId = node.dataset.id;
        if (nodeId && nodeMap[nodeId]) {
            nodeMap[nodeId].x = x;
            nodeMap[nodeId].y = y;

            // Update only edges connected to this node
            updateNodeEdges(nodeId);
        }
    }

    // Update edges for a specific node
    function updateNodeEdges(nodeId: string) {
        if (edgesByNode[nodeId]) {
            edgesByNode[nodeId].forEach((edgeId) => {
                const edge = edges.find((e) => e.id === edgeId);
                if (edge) {
                    updateEdge(edge);
                }
            });
        }
    }

    // Update a single edge
    function updateEdge(edge: {
        id: string;
        source: string;
        target: string;
        element: SVGPathElement;
    }) {
        const sourceNode = nodeMap[edge.source];
        const targetNode = nodeMap[edge.target];

        if (sourceNode && targetNode) {
            // Calculate center points
            const sourceX = sourceNode.x + 50;
            const sourceY = sourceNode.y + 50;
            const targetX = targetNode.x + 50;
            const targetY = targetNode.y + 50;

            // Draw path
            edge.element.setAttribute(
                "d",
                `M ${sourceX} ${sourceY} L ${targetX} ${targetY}`,
            );
        }
    }

    // Update all edges - used only when necessary
    function updateAllEdges() {
        edges.forEach((edge) => updateEdge(edge));
    }

    // Create an edge between two nodes
    function createEdge(
        sourceId: string,
        targetId: string,
        fragment: DocumentFragment | null = null,
        updateNow = true,
    ) {
        // Check if edge already exists
        if (
            edges.some(
                (e) =>
                    (e.source === sourceId && e.target === targetId) ||
                    (e.source === targetId && e.target === sourceId),
            )
        ) {
            return null;
        }

        const edgeElement = document.createElementNS(
            "http://www.w3.org/2000/svg",
            "path",
        );
        const edgeId = `edge-${edges.length}`;
        const edge = {
            id: edgeId,
            source: sourceId,
            target: targetId,
            element: edgeElement,
        };

        edgeElement.id = edgeId;
        edgeElement.classList.add("edge");

        // Add to fragment if provided, otherwise directly to edgesLayer
        if (fragment) {
            fragment.appendChild(edgeElement);
        } else {
            edgesLayer.appendChild(edgeElement);
        }

        edges = [...edges, edge];

        // Add to edge lookup maps
        if (!edgesByNode[sourceId]) edgesByNode[sourceId] = [];
        if (!edgesByNode[targetId]) edgesByNode[targetId] = [];

        edgesByNode[sourceId].push(edgeId);
        edgesByNode[targetId].push(edgeId);

        if (updateNow) {
            updateEdge(edge);
        }

        return edge;
    }

    // Throttled edge updates for continuous operations
    function throttledUpdateEdges() {
        if (!edgeUpdatePending) {
            edgeUpdatePending = true;
            requestAnimationFrame(() => {
                updateAllEdges();
                edgeUpdatePending = false;
            });
        }
    }

    // Handle wheel event for zooming
    function handleWheel(e: WheelEvent) {
        e.preventDefault();

        const rect = canvasContainer.getBoundingClientRect();
        const mouseX = e.clientX - rect.left;
        const mouseY = e.clientY - rect.top;

        // Calculate position before zoom
        const beforeZoomX = (mouseX - posX) / scale;
        const beforeZoomY = (mouseY - posY) / scale;

        // Adjust scale
        if (e.deltaY < 0) {
            // Zoom in
            scale *= 1.1;
            if (scale > 5) scale = 5;
        } else {
            // Zoom out
            scale /= 1.1;
            if (scale < 0.2) scale = 0.2;
        }

        // Adjust position to zoom at mouse position
        posX = mouseX - beforeZoomX * scale;
        posY = mouseY - beforeZoomY * scale;

        updateTransform();
    }

    // Reset view
    function resetView() {
        scale = 1;
        posX = 0;
        posY = 0;
        updateTransform();
    }

    // Reset everything
    function resetAll() {
        resetView();

        // Remove all nodes and edges
        nodes.forEach((node) => {
            if (node.element.parentNode) {
                node.element.parentNode.removeChild(node.element);
            }
        });

        edges.forEach((edge) => {
            if (edge.element.parentNode) {
                edge.element.parentNode.removeChild(edge.element);
            }
        });

        nodes = [];
        edges = [];
        nodeMap = {};
        edgesByNode = {};

        // Clear any temporary lines
        const tempLine = document.getElementById("temp-line");
        if (tempLine && tempLine.parentNode) {
            tempLine.parentNode.removeChild(tempLine);
        }

        // Recreate initial nodes
        createInitialNodes();
    }

    // Set mode
    function setMode(mode: string) {
        currentMode = mode;
    }

    // Handle mouse move
    function handleMouseMove(e: MouseEvent) {
        if (isPanning) {
            // Handle panning
            posX = e.clientX - startX;
            posY = e.clientY - startY;
            updateTransform();
        } else if (isDraggingNode && currentNode) {
            // Handle node dragging
            e.preventDefault();

            // Calculate position in canvas space
            const canvasRect = canvasContainer.getBoundingClientRect();

            // Calculate the mouse position in the original (unscaled) canvas coordinate system
            const mouseXInCanvas = (e.clientX - canvasRect.left - posX) / scale;
            const mouseYInCanvas = (e.clientY - canvasRect.top - posY) / scale;

            // Apply the original offset, also scaled appropriately
            const x = mouseXInCanvas - nodeOffsetX / scale;
            const y = mouseYInCanvas - nodeOffsetY / scale;

            updateNodePosition(currentNode, x, y);
        } else if (currentMode === "connect" && sourceNode) {
            // Handle connection line drawing
            const tempLine = document.getElementById("temp-line");
            if (tempLine) {
                const canvasRect = canvasContainer.getBoundingClientRect();
                const sourceNodePos = getNodeCenter(sourceNode);

                // Calculate endpoint in canvas coordinate space
                const endX =
                    (e.clientX - canvasRect.left) / scale - posX / scale;
                const endY =
                    (e.clientY - canvasRect.top) / scale - posY / scale;

                tempLine.setAttribute("x2", endX.toString());
                tempLine.setAttribute("y2", endY.toString());
            }
        }
    }

    // Handle mouse up
    function handleMouseUp(e: MouseEvent) {
        if (isPanning) {
            isPanning = false;
            canvasContainer.style.cursor = "default";
        }

        if (isDraggingNode) {
            isDraggingNode = false;
            if (currentNode) {
                // Remove Tailwind classes for selected state
                currentNode.classList.remove("ring-2", "ring-yellow-400");
                currentNode = null;
            }
        }

        if (currentMode === "connect" && sourceNode) {
            // Check if we're over another node
            let targetNode = null;

            // Find if there's a node under the mouse
            if (
                e.target instanceof HTMLElement &&
                e.target.classList &&
                e.target.classList.contains("node")
            ) {
                targetNode = e.target;
            }

            // Remove temporary line
            const tempLine = document.getElementById("temp-line");
            if (tempLine && tempLine.parentNode) {
                tempLine.parentNode.removeChild(tempLine);
            }

            // Create connection if target is valid
            if (targetNode && targetNode !== sourceNode) {
                createEdge(sourceNode.dataset.id!, targetNode.dataset.id!);
            }

            sourceNode = null;
        }
    }

    // Handle middle mouse button for panning
    function handleCanvasMouseDown(e: MouseEvent) {
        if (e.button === 1) {
            // Middle mouse button
            e.preventDefault();
            isPanning = true;
            startX = e.clientX - posX;
            startY = e.clientY - posY;
            canvasContainer.style.cursor = "grabbing";
        } else if (e.button === 0 && e.target === canvas) {
            // Left click on empty canvas area
            // You could add functionality here if needed
        }
    }

    // Prevent context menu on middle click
    function handleContextMenu(e: MouseEvent) {
        if (e.button === 1) {
            // Middle button
            e.preventDefault();
        }
    }
</script>

<svelte:window on:mousemove={handleMouseMove} on:mouseup={handleMouseUp} />

<div class="w-full min-h-screen bg-gray-100">
    <div class="w-full h-screen mx-auto flex flex-col items-center">
        <div
            class="absolute z-50 flex gap-2 flex-wrap justify-center mb-5 w-full"
        >
            <button
                class="px-4 py-2 bg-green-600 text-white rounded transition-colors {currentMode ===
                'move'
                    ? 'ring-2 ring-green-800 bg-green-700'
                    : 'hover:bg-green-700'}"
                onclick={() => setMode("move")}
            >
                Move Mode
            </button>
            <button
                class="px-4 py-2 bg-green-600 text-white rounded transition-colors {currentMode ===
                'connect'
                    ? 'ring-2 ring-green-800 bg-green-700'
                    : 'hover:bg-green-700'}"
                onclick={() => setMode("connect")}
            >
                Connect Mode
            </button>
            <button
                class="px-4 py-2 bg-green-600 text-white rounded hover:bg-green-700 transition-colors"
                onclick={resetView}
            >
                Reset View
            </button>
            <button
                class="px-4 py-2 bg-green-600 text-white rounded hover:bg-green-700 transition-colors"
                onclick={resetAll}
            >
                Reset All
            </button>
            <button
                class="px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700 transition-colors"
                onclick={createStressTest}
            >
                Stress Test
            </button>
        </div>

        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
            class="w-full h-screen overflow-hidden relative bg-gray-100"
            bind:this={canvasContainer}
            onmousedown={handleCanvasMouseDown}
            oncontextmenu={handleContextMenu}
            onwheel={handleWheel}
        >
            <div
                class="w-full h-full relative origin-top-left"
                bind:this={canvas}
            >
                <svg
                    class="absolute top-0 left-0 w-full h-full pointer-events-none"
                    bind:this={edgesLayer}
                    viewBox="0 0 100% 100%"
                    preserveAspectRatio="none"
                    style="overflow: visible;"
                ></svg>

                <!-- Nodes will be added here dynamically -->
            </div>
        </div>
    </div>
</div>

<style>
    /* SVG animation for edges */
    @keyframes flowAnimation {
        0% {
            stroke-dashoffset: 24;
        }
        100% {
            stroke-dashoffset: 0;
        }
    }

    :global(.edge) {
        stroke: #2196f3;
        stroke-width: 3;
        stroke-dasharray: 12 12;
        fill: none;
        animation: flowAnimation 1s linear infinite;
    }
</style>
