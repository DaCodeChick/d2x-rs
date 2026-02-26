#include "render/MineRenderer.h"
#include "core/types/Types.h"
#include <QFile>
#include <QDir>
#include <format>
#include <print>

// Initialize resources before entering namespace
// This must be at global scope for Q_INIT_RESOURCE to work correctly
static struct ResourceInitializer {
    ResourceInitializer() {
        Q_INIT_RESOURCE(shaders);
    }
} _resourceInit;

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
    // Pipeline creation is deferred until first render when we have a render target
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

    // Create pipeline on first render when we have a render target
    if (!m_pipeline) {
        createPipeline(renderTarget);
    }

    if (!m_pipeline || !m_vertexBuffer || !m_indexBuffer) {
        return;
    }

    // Update uniform buffer with MVP matrix
    QMatrix4x4 mvpMatrix = m_projectionMatrix * m_viewMatrix;
    QRhiResourceUpdateBatch* resourceUpdates = m_rhi->nextResourceUpdateBatch();
    resourceUpdates->updateDynamicBuffer(m_uniformBuffer.get(), 0, 64, mvpMatrix.constData());
    cb->resourceUpdate(resourceUpdates);

    // Set up graphics pipeline state
    cb->setGraphicsPipeline(m_pipeline.get());
    cb->setShaderResources(m_srb.get());
    cb->setViewport(QRhiViewport(0, 0, renderTarget->pixelSize().width(), renderTarget->pixelSize().height()));

    // Bind vertex and index buffers
    const QRhiCommandBuffer::VertexInput vertexBindings[] = {
        { m_vertexBuffer.get(), 0 }
    };
    cb->setVertexInput(0, 1, vertexBindings, m_indexBuffer.get(), 0, QRhiCommandBuffer::IndexUInt16);

    // Draw the wireframe
    cb->drawIndexed(static_cast<quint32>(m_indices.size()));
}

void MineRenderer::createShaders() {
    // Shaders are loaded as part of pipeline creation from .qsb files
    std::println("MineRenderer: Shaders will be loaded from .qsb files");
}

void MineRenderer::createPipeline(QRhiRenderTarget* renderTarget) {
    if (!m_rhi || !renderTarget) {
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

    // Load shaders from .qsb files
    QShader vertShader = QShader::fromSerialized(loadShaderFile(":/shaders/basic.vert.qsb"));
    QShader fragShader = QShader::fromSerialized(loadShaderFile(":/shaders/basic.frag.qsb"));
    
    if (!vertShader.isValid()) {
        std::println(stderr, "Failed to load vertex shader");
        return;
    }
    if (!fragShader.isValid()) {
        std::println(stderr, "Failed to load fragment shader");
        return;
    }

    // Create graphics pipeline
    m_pipeline.reset(m_rhi->newGraphicsPipeline());
    
    // Set up pipeline state
    m_pipeline->setShaderStages({
        { QRhiShaderStage::Vertex, vertShader },
        { QRhiShaderStage::Fragment, fragShader }
    });

    // Vertex input layout: position (vec3) + color (vec4)
    QRhiVertexInputLayout inputLayout;
    inputLayout.setBindings({
        { 7 * sizeof(float) }  // stride: 3 floats (pos) + 4 floats (color)
    });
    inputLayout.setAttributes({
        { 0, 0, QRhiVertexInputAttribute::Float3, 0 },                    // position at offset 0
        { 0, 1, QRhiVertexInputAttribute::Float4, 3 * sizeof(float) }     // color at offset 12
    });
    m_pipeline->setVertexInputLayout(inputLayout);

    m_pipeline->setShaderResourceBindings(m_srb.get());
    m_pipeline->setRenderPassDescriptor(renderTarget->renderPassDescriptor());
    m_pipeline->setTopology(QRhiGraphicsPipeline::Lines);  // Wireframe rendering

    if (!m_pipeline->create()) {
        std::println(stderr, "Failed to create graphics pipeline");
        return;
    }

    std::println("MineRenderer: Pipeline created successfully");
}

QByteArray MineRenderer::loadShaderFile(const QString& filename) {
    QFile file(filename);
    if (!file.open(QIODevice::ReadOnly)) {
        std::println(stderr, "Failed to open shader file: {}", filename.toStdString());
        return QByteArray();
    }
    return file.readAll();
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
