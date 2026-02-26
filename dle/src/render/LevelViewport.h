#pragma once

#include <QRhiWidget>
#include <QColor>
#include <memory>

namespace dle {

/**
 * @brief 3D viewport widget for rendering Descent levels using Qt RHI
 * 
 * This widget uses Qt's Render Hardware Interface (RHI) for cross-platform
 * graphics rendering (Metal/Vulkan/D3D11/D3D12/OpenGL).
 * 
 * Currently renders a simple clear color as a proof of concept.
 * Future enhancements will add:
 * - Camera controls (WASD + mouse look)
 * - Mine geometry rendering
 * - Segment selection and highlighting
 */
class LevelViewport : public QRhiWidget {
    Q_OBJECT

public:
    explicit LevelViewport(QWidget* parent = nullptr);
    ~LevelViewport() override = default;

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
    // Background clear color (dark gray)
    QColor m_clearColor{51, 51, 51}; // RGB: (51, 51, 51) ≈ 0.2 * 255
};

} // namespace dle
