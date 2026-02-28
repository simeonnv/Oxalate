<template>
    <div  v-if="results" ref="containerRef" class="w-full h-full flex flex-row items-center justify-center overflow-hidden bg-base-300/60">
      <svg ref="svgRef" width="100%" height="100%"></svg>
    </div>

    <div v-else class="w-full h-full text-4xl font-bold flex items-center justify-center">
        Failed to load graph
    </div>
</template>

<script setup lang="ts">
definePageMeta({
    layout: 'search-bar',
})

import { onMounted, ref, computed, watch } from 'vue';
import { useRoute } from 'vue-router';
import * as d3 from 'd3';

interface ApiNode {
  word: string;
  usage: number;
}

interface ApiRelation {
  source_word: string;
  weight: number;
  target_word: string;
}

interface ApiResponse {
  nodes: ApiNode[];
  relations: ApiRelation[];
}

interface GraphNode extends d3.SimulationNodeDatum {
  id: string;
  size: number;
}

interface GraphLink extends d3.SimulationLinkDatum<GraphNode> {
  source: string | GraphNode;
  target: string | GraphNode;
  power: number;
}

const route = useRoute();
const query = computed(() => (route.query.q as string) ?? '');

const { data: results, pending, error } = await useFetch<ApiResponse>('http://localhost:22267/keyword_graph', {
  method: 'POST',
  body: computed(() => ({
    text: query.value
  })),
  watch: [query],
  immediate: !!query.value,
});

const svgRef = ref<SVGSVGElement | null>(null);
const containerRef = ref<HTMLDivElement | null>(null);

const width = ref(0);
const height = ref(0);

let simulation: d3.Simulation<GraphNode, undefined> | null = null;

const updateDimensions = () => {
  if (containerRef.value) {
    width.value = containerRef.value.clientWidth;
    height.value = containerRef.value.clientHeight;
    
    if (simulation) {
      simulation.force("center", d3.forceCenter(width.value / 2, height.value / 2));
      simulation.alpha(0.3).restart(); // Give it a little nudge to re-center
    }
  }
};

const renderGraph = (apiData: ApiResponse) => {
  if (!svgRef.value || width.value === 0) return;
  
  const svg = d3.select(svgRef.value);
  svg.selectAll("*").remove();
  if (simulation) simulation.stop();

  const container = svg.append("g");

  const zoom = d3.zoom<SVGSVGElement, unknown>()
    .scaleExtent([0.1, 8])
    .on("zoom", (event) => {
      container.attr("transform", event.transform); 
    });

  svg.call(zoom as any);

  const nodes: GraphNode[] = apiData.nodes.map(n => ({
    id: n.word,
    size: n.usage
  }));
  
  const links: GraphLink[] = apiData.relations.map(r => ({
    source: r.source_word,
    target: r.target_word,
    power: r.weight
  }));

  const maxUsage = d3.max(nodes, d => d.size) || 1;
  const radiusScale = d3.scaleSqrt().domain([0, maxUsage]).range([30, 50]);

  
  const [minWeight, maxWeight] = d3.extent(links, d => d.power) as [number, number];

  const linkScale = d3.scaleLinear()
    .domain([minWeight || 0, maxWeight || 10])
    .range([2, 20]); 

  const opacityScale = d3.scaleLinear()
    .domain([minWeight || 0, maxWeight || 10])
    .range([0.2, 0.8]); 

  simulation = d3.forceSimulation<GraphNode>(nodes)
    .force("link", d3.forceLink<GraphNode, GraphLink>(links).id(d => d.id).distance(200))
    .force("charge", d3.forceManyBody().strength(-600))
    .force("center", d3.forceCenter(width.value / 2, height.value / 2));


  const linkBg = container.append("g")
    .selectAll("line")
    .data(links)
    .join("line")
    .attr("class", "stroke-primary")
    .attr("stroke-width", d => linkScale(d.power) + 2) 
    .attr("stroke-opacity", 0.5);

const link = container.append("g")
  .selectAll("line")
  .data(links)
  .join("line")
  .attr("class", "stroke-primary")
  .attr("stroke-width", d => linkScale(d.power))
  .attr("stroke-opacity", d => opacityScale(d.power))
  .attr("stroke-linecap", "round");

const linkLabels = container.append("g")
  .selectAll("text")
  .data(links)
  .join("text")
  .text(d => d.power.toFixed(1))
  .attr("class", "fill-secondary-content text-xs select-none pointer-events-none")
  .attr("text-anchor", "middle")
  .attr("dominant-baseline", "central")
  .style("background", "white"); 

  const node = container.append("g")
    .selectAll("circle")
    .data(nodes)
    .join("circle")
    .attr("r", d => radiusScale(d.size))
    .attr("class", "fill-primary")
    .style("cursor", "grab")
    .call(drag(simulation) as any); 
    
  const labels = container.append("g")
    .selectAll("text")
    .data(nodes)
    .join("text")
    .text(d => d.id)
    .attr("class", "fill-primary-content text-md font-bold select-none pointer-events-none")
    .attr("text-anchor", "middle")      
    .attr("dominant-baseline", "central");

  simulation.on("tick", () => {
    linkBg
      .attr("x1", d => (d.source as any).x)
      .attr("y1", d => (d.source as any).y)
      .attr("x2", d => (d.target as any).x)
      .attr("y2", d => (d.target as any).y);

    link
      .attr("x1", d => (d.source as any).x)
      .attr("y1", d => (d.source as any).y)
      .attr("x2", d => (d.target as any).x)
      .attr("y2", d => (d.target as any).y);

    linkLabels
      .attr("x", d => {
        const source = d.source as any;
        const target = d.target as any;
        return (source.x + target.x) / 2;
      })
      .attr("y", d => {
        const source = d.source as any;
        const target = d.target as any;
        return (source.y + target.y) / 2;
      });
        
    node
      .attr("cx", d => d.x ?? 0)
      .attr("cy", d => d.y ?? 0);
      
    labels
      .attr("x", d => d.x ?? 0)
      .attr("y", d => d.y ?? 0);
  });
    
  function drag(sim: d3.Simulation<GraphNode, undefined>) {
    return d3.drag<SVGCircleElement, GraphNode>()
      .on("start", (event, d) => {
        if (!event.active) sim.alphaTarget(0.3).restart();
        d.fx = d.x;
        d.fy = d.y;
      })
      .on("drag", (event, d) => {
        d.fx = event.x;
        d.fy = event.y;
      })
      .on("end", (event, d) => {
        if (!event.active) sim.alphaTarget(0);
        d.fx = null;
        d.fy = null;
      });
  }
};

onMounted(() => {
  updateDimensions();

  const resizeObserver = new ResizeObserver(() => {
    updateDimensions();
  });
  
  if (containerRef.value) {
    resizeObserver.observe(containerRef.value);
  }

  watch(results, (newData) => {
    if (newData?.nodes && newData?.relations) {
      renderGraph(newData);
    }
  }, { immediate: true });
});
</script>
