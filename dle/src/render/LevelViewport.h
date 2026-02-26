#pragma once

#include <QRhiWidget>
#include <QColor>
#include <QMatrix4x4>
#include <memory>

namespace dle {

// Forward declarations
class Mine;
class MineRenderer;

/**
 * @brief 3D viewport widget for rendering Descent levels using Qt RHI
 * 
 * This widget uses Qt's Render Hardware Interface (RHI) for cross-platform
 * graphics rendering (Metal/Vulkan/D3D11/D3D12/OpenGL).
 * 
 * Features:
 * - Mine geometry rendering (wireframe/solid)
 * - Camera controls (WASD + mouse look)
 * - Segment selection and highlighting
 */
class LevelViewport : public QRhiWidget {
    Q_OBJECT

public:
    explicit LevelViewport(QWidget* parent = nullptr);
    ~LevelViewport() override;

    /**
     * @brief Set the mine to display
     */
    void setMine(const Mine* mine);

    /**
     * @brief Enable/disable wireframe mode
     */
    void setWireframeMode(bool enabled);
    bool isWireframeMode() const;

protected:
    /**
     * @brief Initialize RHI resources (buffers, pipelines, etc.)
     * Called once when RHI is ready
     */
    void initialize(QRhiCommandBuffer* cb) override;

    /**
     * @brief Render a frame
     * Called whenever the widget needs to be repainted
     */
    void render(QRhiCommandBuffer* cb) override;

private:
    void updateCamera();

    // Renderer
    std::unique_ptr<MineRenderer> m_renderer;

    // Camera
    QMatrix4x4 m_viewMatrix;
    QMatrix4x4 m_projectionMatrix;
    float m_cameraDistance = 50.0f;
    float m_cameraYaw = 0.0f;
    float m_cameraPitch = 0.0f;

    // Background clear color (dark gray)
    QColor m_clearColor{51, 51, 51}; // RGB: (51, 51, 51) ≈ 0.2 * 255
};

} // namespace dle
