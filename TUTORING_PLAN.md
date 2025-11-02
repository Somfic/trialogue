# Procedural Planet Rendering - Tutoring Plan

## Project Goal
Build a real-time procedurally generated planet renderer with seamless space-to-surface transitions.

## Student Profile
- **Experience Level**: Intermediate - comfortable with graphics programming basics
- **Focus Areas**: All aspects (terrain topology, visual appearance, procedural generation, performance/LOD)
- **Primary Interest**: Space-to-surface seamless planet rendering

---

## Learning Path Overview

This is a comprehensive 10-week plan to build a real-time procedural planet system. Each phase builds on the previous one.

---

### **Phase 1: Sphere Foundation** (Week 1-2)
**Status**: 🔲 Not Started | **Current Phase**: ✅

**Learning Objectives:**
- Understand cube sphere vs icosphere topology
- Learn why cube spheres work better for LOD and texturing
- Master sphere coordinate systems and UV mapping

**Topics to Cover:**
- Cube sphere generation (6 faces, normalized to sphere)
- Vertex generation and indexing
- UV coordinate mapping on sphere faces
- Basic sphere rendering

**Implementation Tasks:**
1. Create sphere mesh generation code (start with low subdivision)
2. Implement proper UV mapping
3. Write basic shader to visualize the sphere
4. Verify normals are correct

**Resources to Share:**
- Cube sphere vs icosphere comparison diagrams
- Code examples of normalized cube generation
- UV mapping techniques for cube spheres

**Key Concepts:**
- Why normalized cube = sphere
- Edge/corner handling in cube sphere
- Subdivision levels and vertex count

---

### **Phase 2: Procedural Noise** (Week 2-3)
**Status**: 🔲 Not Started

**Learning Objectives:**
- Master noise fundamentals (Perlin, Simplex, Value noise)
- Understand Fractal Brownian Motion (fBm)
- Learn to compose noise for natural-looking features
- GPU vs CPU noise generation tradeoffs

**Topics to Cover:**
- Noise function theory and implementation
- Octaves, lacunarity, persistence parameters
- Domain warping for organic shapes
- Noise performance considerations

**Implementation Tasks:**
1. Implement Simplex noise (shader or CPU, discuss tradeoffs)
2. Create fBm function with configurable parameters
3. Apply noise to displace sphere vertices
4. Experiment with parameter tuning for different terrain types

**Resources to Share:**
- Noise function visualizations
- fBm parameter guides
- Domain warping examples

**Key Concepts:**
- Gradient vs value noise
- Frequency/amplitude relationship
- Multi-octave composition
- 3D noise on sphere surface

---

### **Phase 3: Level of Detail System** (Week 3-5)
**Status**: 🔲 Not Started

**Learning Objectives:**
- Understand quadtree spatial subdivision
- Learn LOD principles for large-scale rendering
- Master crack prevention techniques
- Implement efficient chunk management

**Topics to Cover:**
- Quadtree LOD on cube sphere faces
- Distance-based subdivision criteria
- T-junction prevention (stitching/skirts)
- Frustum culling and chunk streaming

**Implementation Tasks:**
1. Create quadtree data structure for each cube face
2. Implement distance-based subdivision logic
3. Add stitching geometry or skirts to prevent cracks
4. Implement chunk loading/unloading system
5. Add frustum culling for performance

**Resources to Share:**
- Quadtree visualization tools
- LOD distance calculation formulas
- T-junction problem and solutions
- Chunked LOD terrain papers

**Key Concepts:**
- Spatial subdivision strategies
- Screen-space error metrics
- Neighbor LOD relationships
- Memory management for chunks

---

### **Phase 4: Advanced Terrain Features** (Week 5-7)
**Status**: 🔲 Not Started

**Learning Objectives:**
- Layer multiple noise scales for realistic terrain
- Create biome systems with smooth transitions
- Implement procedural texturing techniques
- Generate normal maps from heightmaps

**Topics to Cover:**
- Multi-scale noise composition (continental, regional, local)
- Biome generation (moisture/temperature maps)
- Height-based and slope-based coloring
- Triplanar texture mapping
- Runtime normal map generation

**Implementation Tasks:**
1. Layer continent-scale + detail noise
2. Create biome blending system
3. Implement height and slope-based coloring
4. Add triplanar texture mapping (avoids UV distortion)
5. Generate normals from heightmap data

**Resources to Share:**
- Real-world terrain analysis
- Biome classification systems
- Triplanar mapping technique
- Normal map generation from heightfields

**Key Concepts:**
- Scale separation in terrain
- Gradient-based biome transitions
- Texture coordinate-free mapping
- Derivative-based normal calculation

---

### **Phase 5: Atmospheric Scattering** (Week 7-8)
**Status**: 🔲 Not Started

**Learning Objectives:**
- Understand light scattering physics
- Implement Rayleigh and Mie scattering
- Create realistic day/night cycles
- Optimize atmospheric shaders

**Topics to Cover:**
- Rayleigh scattering (blue sky, wavelength dependency)
- Mie scattering (sunset colors, atmospheric haze)
- Optical depth calculation
- Space-to-surface atmospheric transition
- Pre-computed lookup table optimization

**Implementation Tasks:**
1. Implement atmospheric scattering shader
2. Add day/night cycle support
3. Create horizon glow and limb darkening
4. Optimize with pre-computed lookup tables

**Resources to Share:**
- Atmospheric scattering papers (Bruneton, O'Neil)
- Scattering visualization diagrams
- Performance optimization techniques
- Real atmospheric reference images

**Key Concepts:**
- Light interaction with atmosphere
- Ray marching through atmosphere
- Optical depth integration
- Lookup table generation

---

### **Phase 6: Lighting & Polish** (Week 8-9)
**Status**: 🔲 Not Started

**Learning Objectives:**
- Implement proper terrain lighting
- Add shadow systems at planetary scale
- Create convincing surface details
- Optional: Ocean rendering

**Topics to Cover:**
- Directional lighting for terrain
- Normal mapping from procedural heightmaps
- Large-scale shadow techniques
- Horizon-based ambient occlusion
- Optional: Ocean shader with waves

**Implementation Tasks:**
1. Add lighting system for terrain
2. Implement shadow mapping or HBAO
3. Enhance surface detail with lighting
4. Optional: Simple ocean shader

**Resources to Share:**
- Terrain lighting techniques
- Planetary-scale shadow papers
- HBAO explanations
- Ocean rendering tutorials

**Key Concepts:**
- Light and shadow at scale
- Self-shadowing terrain
- Ambient occlusion approximations
- Specular highlights on terrain

---

### **Phase 7: Performance & Optimization** (Week 9-10)
**Status**: 🔲 Not Started

**Learning Objectives:**
- Profile GPU performance effectively
- Optimize chunk generation pipeline
- Use compute shaders for terrain
- Manage memory for large-scale worlds

**Topics to Cover:**
- GPU profiling techniques
- Async/threaded chunk generation
- Compute shader terrain generation
- Memory management strategies
- Draw call optimization

**Implementation Tasks:**
1. Implement GPU-side noise and mesh generation
2. Add background chunk loading
3. Create performance monitoring
4. Optimize based on profiling results

**Resources to Share:**
- GPU profiling tools (RenderDoc, PIX, etc.)
- Compute shader examples
- Async processing patterns
- Memory profiling techniques

**Key Concepts:**
- GPU bottleneck identification
- CPU-GPU parallelization
- Memory bandwidth optimization
- Frame time budgeting

---

## Teaching Methodology

### For Each Phase:

1. **Concept Explanation**
   - Explain the theory with diagrams/examples
   - Show real-world references
   - Discuss the "why" behind techniques

2. **Approach Discussion**
   - Discuss architecture options
   - Explain tradeoffs of different approaches
   - Let student choose direction (with guidance)

3. **Implementation Guidance**
   - Student implements with tutor guidance
   - Provide hints, not complete solutions
   - Encourage experimentation

4. **Code Review**
   - Review student's implementation
   - Suggest improvements and alternatives
   - Explain best practices

5. **Debugging Assistance**
   - Help diagnose issues
   - Teach debugging techniques
   - Build problem-solving skills

6. **Optimization Discussion**
   - Analyze performance
   - Suggest optimizations
   - Explain measurement techniques

### Tutoring Principles:

- **Don't write complete code** - guide the student to implement themselves
- **Explain tradeoffs** - help them make informed decisions
- **Build understanding** - theory before implementation
- **Encourage exploration** - it's okay to try different approaches
- **Be patient with fundamentals** - ensure solid understanding before moving on

---

## Session Notes

### Session 1 (2025-11-02)
- Explored current codebase
- Identified project uses wgpu (WebGPU) with Bevy ECS
- Has existing raytracer and rasterizer layers
- Student wants space-to-surface procedural planet
- Interested in: terrain, visuals, procedural generation, and LOD
- Created tutoring plan
- **Next Steps**: Begin Phase 1 - Sphere Foundation

---

## Progress Tracker

| Phase | Status | Start Date | Complete Date | Notes |
|-------|--------|------------|---------------|-------|
| Phase 1: Sphere Foundation | Not Started | - | - | Ready to begin |
| Phase 2: Procedural Noise | Not Started | - | - | - |
| Phase 3: Level of Detail | Not Started | - | - | - |
| Phase 4: Terrain Features | Not Started | - | - | - |
| Phase 5: Atmosphere | Not Started | - | - | - |
| Phase 6: Lighting & Polish | Not Started | - | - | - |
| Phase 7: Performance | Not Started | - | - | - |

---

## Quick Reference for Future Sessions

**Current Phase**: Phase 1 - Sphere Foundation
**Last Topic Covered**: Project planning
**Next Topic**: Cube sphere generation technique

**Context for AI Tutor**:
- Student has working engine with wgpu/bevy
- Comfortable with intermediate graphics concepts
- Wants guidance, not complete solutions
- Focus on real-time performance for game applications
- Building toward space-to-surface procedural planet

**Code Locations**:
- Engine: `/home/lucas/trialogue/crates/engine/`
- Shaders: `/home/lucas/trialogue/crates/engine/src/layers/*/`
- Components: `/home/lucas/trialogue/crates/engine/src/components/`

**When Resuming**:
1. Check "Session Notes" for last session summary
2. Check "Progress Tracker" for current phase status
3. Review last completed tasks
4. Begin with brief recap before continuing
