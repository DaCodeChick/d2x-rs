#include "render/MineRenderer.h"
#include "core/types/Types.h"
#include <QFile>
#include <format>
#include <print>

namespace dle {

MineRenderer::MineRenderer() {
    // Initialize identity matrices
    m_viewMatrix.setToIdentity();
    m_projectionMatrix.setToIdentity();
}

MineRenderer::~MineRenderer() {
    cleanup();
}

void MineRenderer::initialize(QRhi* rhi) {
    m_rhi = rhi;
    
    if (!m_rhi) {
        std::println(stderr, "MineRenderer::initialize: RHI is null");
        return;
    }

    createShaders();
    createPipeline();
}

void MineRenderer::cleanup() {
    m_pipeline.reset();
    m_srb.reset();
    m_uniformBuffer.reset();
    m_indexBuffer.reset();
    m_vertexBuffer.reset();
    m_rhi = nullptr;
}

void MineRenderer::setMine(const Mine* mine) {
    m_mine = mine;
    m_needsBufferUpdate = true;
}

void MineRenderer::updateBuffers() {
    if (!m_mine || !m_rhi) {
        return;
    }

    if (m_wireframeMode) {
        buildWireframeBuffers();
    } else {
        buildMeshBuffers();
    }

    m_needsBufferUpdate = false;
}

void MineRenderer::render(QRhiCommandBuffer* cb, QRhiRenderTarget* renderTarget) {
    if (!m_mine || !m_rhi || !m_pipeline || m_indices.empty()) {
        return;
    }

    if (m_needsBufferUpdate) {
        updateBuffers();
    }

    // TODO: Implement actual rendering
    // Will need to:
    // 1. Update uniform buffer with view/projection matrices
    // 2. Bind pipeline
    // 3. Bind vertex/index buffers
    // 4. Draw indexed primitives
}

void MineRenderer::createShaders() {
    // TODO: Create GLSL shaders and compile to SPIR-V
    // For now, just placeholder
    std::println("MineRenderer: Creating shaders...");
}

void MineRenderer::createPipeline() {
    // TODO: Create graphics pipeline
    // For now, just placeholder
    std::println("MineRenderer: Creating pipeline...");
}

void MineRenderer::buildWireframeBuffers() {
    if (!m_mine) {
        return;
    }

    m_vertices.clear();
    m_indices.clear();

    const auto& vertices = m_mine->getVertices();
    const auto& segments = m_mine->getSegments();

    // Convert vertices to GPU format
    m_vertices.reserve(vertices.size());
    for (const auto& vertex : vertices) {
        // Convert from fixed-point to float
        Vertex v;
        v.x = static_cast<float>(fixToDouble(vertex.position.x));
        v.y = static_cast<float>(fixToDouble(vertex.position.y));
        v.z = static_cast<float>(fixToDouble(vertex.position.z));
        v.r = 1.0f;  // White color for wireframe
        v.g = 1.0f;
        v.b = 1.0f;
        v.a = 1.0f;
        m_vertices.push_back(v);
    }

    // Build indices for edges (wireframe)
    // Each segment has 12 edges
    for (const auto& segment : segments) {
        const auto& vertexIds = segment.getVertexIds();
        
        // Use EDGE_VERTEX_TABLE to get the 12 edges
        for (int edgeIdx = 0; edgeIdx < 12; ++edgeIdx) {
            uint16_t v0 = vertexIds[EDGE_VERTEX_TABLE[edgeIdx][0]];
            uint16_t v1 = vertexIds[EDGE_VERTEX_TABLE[edgeIdx][1]];
            m_indices.push_back(v0);
            m_indices.push_back(v1);
        }
    }

    std::println("MineRenderer: Built wireframe buffers - {} vertices, {} indices",
                 m_vertices.size(), m_indices.size());
}

void MineRenderer::buildMeshBuffers() {
    if (!m_mine) {
        return;
    }

    // TODO: Build solid mesh with triangulated faces
    std::println("MineRenderer: Building mesh buffers (not yet implemented)");
}

} // namespace dle
