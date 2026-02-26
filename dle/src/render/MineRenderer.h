#pragma once

#include "core/mine/Mine.h"
#include <rhi/qrhi.h>
#include <QMatrix4x4>
#include <memory>
#include <vector>

namespace dle {

/**
 * @brief Renders Descent mine geometry using Qt RHI
 * 
 * Features:
 * - Wireframe rendering of segment edges
 * - Solid face rendering
 * - Configurable camera position/orientation
 * 
 * Rendering approach:
 * - Convert mine segments to GPU vertex/index buffers
 * - Use simple shader for wireframe/solid rendering
 * - Support for view/projection transformations
 */
class MineRenderer {
public:
    MineRenderer();
    ~MineRenderer();

    /**
     * @brief Initialize RHI resources (buffers, pipeline, shaders)
     * Call this once when RHI is ready
     */
    void initialize(QRhi* rhi);

    /**
     * @brief Clean up RHI resources
     */
    void cleanup();

    /**
     * @brief Set the mine data to render
     */
    void setMine(const Mine* mine);

    /**
     * @brief Update GPU buffers with current mine data
     */
    void updateBuffers();

    /**
     * @brief Render the mine
     * @param cb Command buffer for recording draw commands
     * @param renderTarget Current render target
     */
    void render(QRhiCommandBuffer* cb, QRhiRenderTarget* renderTarget);

    /**
     * @brief Set camera transform
     */
    void setViewMatrix(const QMatrix4x4& view) { m_viewMatrix = view; }
    void setProjectionMatrix(const QMatrix4x4& projection) { m_projectionMatrix = projection; }

    /**
     * @brief Enable/disable wireframe mode
     */
    void setWireframeMode(bool enabled) { m_wireframeMode = enabled; }
    bool isWireframeMode() const { return m_wireframeMode; }

private:
    // Vertex structure for GPU
    struct Vertex {
        float x, y, z;      // Position
        float r, g, b, a;   // Color
    };

    void createShaders();
    void createPipeline();
    void buildMeshBuffers();
    void buildWireframeBuffers();

    // RHI resources
    QRhi* m_rhi = nullptr;
    std::unique_ptr<QRhiBuffer> m_vertexBuffer;
    std::unique_ptr<QRhiBuffer> m_indexBuffer;
    std::unique_ptr<QRhiBuffer> m_uniformBuffer;
    std::unique_ptr<QRhiShaderResourceBindings> m_srb;
    std::unique_ptr<QRhiGraphicsPipeline> m_pipeline;

    // Mine data
    const Mine* m_mine = nullptr;
    std::vector<Vertex> m_vertices;
    std::vector<uint16_t> m_indices;
    
    // Camera
    QMatrix4x4 m_viewMatrix;
    QMatrix4x4 m_projectionMatrix;

    // Rendering state
    bool m_wireframeMode = true;
    bool m_needsBufferUpdate = false;
};

} // namespace dle
