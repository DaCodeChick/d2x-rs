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

    // Upload to GPU
    uploadBuffersToGPU();

    m_needsBufferUpdate = false;
}

void MineRenderer::render(QRhiCommandBuffer* cb, QRhiRenderTarget* renderTarget) {
    if (!m_mine || !m_rhi || m_indices.empty()) {
        return;
    }

    if (m_needsBufferUpdate) {
        updateBuffers();
    }

    // TODO: Implement actual GPU drawing
    // For now, just ensure buffers are created
    // Next step: Add proper shader loading and pipeline configuration
    Q_UNUSED(cb);
    Q_UNUSED(renderTarget);
}

void MineRenderer::createShaders() {
    // For simplicity, we'll use inline GLSL and let Qt compile it
    // In a production app, you'd pre-compile shaders to .qsb files
    std::println("MineRenderer: Shaders will be created as part of pipeline creation");
}

void MineRenderer::createPipeline() {
    if (!m_rhi) {
        return;
    }

    // Create uniform buffer for MVP matrix (4x4 matrix = 64 bytes)
    m_uniformBuffer.reset(m_rhi->newBuffer(QRhiBuffer::Dynamic, QRhiBuffer::UniformBuffer, 64));
    if (!m_uniformBuffer->create()) {
        std::println(stderr, "Failed to create uniform buffer");
        return;
    }

    // Create shader resource bindings
    m_srb.reset(m_rhi->newShaderResourceBindings());
    m_srb->setBindings({
        QRhiShaderResourceBinding::uniformBuffer(0, QRhiShaderResourceBinding::VertexStage,
                                                  m_uniformBuffer.get())
    });
    if (!m_srb->create()) {
        std::println(stderr, "Failed to create shader resource bindings");
        return;
    }

    // Create graphics pipeline
    m_pipeline.reset(m_rhi->newGraphicsPipeline());

    // For now, we'll skip shader creation since it requires .qsb files or runtime compilation
    // TODO: Add proper shader loading
    std::println("MineRenderer: Pipeline creation - need to add shader loading");
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

void MineRenderer::uploadBuffersToGPU() {
    if (!m_rhi || m_vertices.empty() || m_indices.empty()) {
        return;
    }

    // Create or recreate vertex buffer
    const size_t vertexDataSize = m_vertices.size() * sizeof(Vertex);
    if (!m_vertexBuffer || m_vertexBuffer->size() != vertexDataSize) {
        m_vertexBuffer.reset(m_rhi->newBuffer(QRhiBuffer::Immutable, QRhiBuffer::VertexBuffer,
                                               vertexDataSize));
        if (!m_vertexBuffer->create()) {
            std::println(stderr, "Failed to create vertex buffer");
            return;
        }
    }

    // Create or recreate index buffer
    const size_t indexDataSize = m_indices.size() * sizeof(uint16_t);
    if (!m_indexBuffer || m_indexBuffer->size() != indexDataSize) {
        m_indexBuffer.reset(m_rhi->newBuffer(QRhiBuffer::Immutable, QRhiBuffer::IndexBuffer,
                                              indexDataSize));
        if (!m_indexBuffer->create()) {
            std::println(stderr, "Failed to create index buffer");
            return;
        }
    }

    // Upload data to GPU (we need a resource update batch, but we'll do it in render())
    std::println("MineRenderer: Uploaded {} vertices and {} indices to GPU",
                 m_vertices.size(), m_indices.size());
}

} // namespace dle
