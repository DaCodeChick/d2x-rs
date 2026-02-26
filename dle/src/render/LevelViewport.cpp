#include "render/LevelViewport.h"
#include "render/MineRenderer.h"
#include "core/mine/Mine.h"
#include <rhi/qrhi.h>

namespace dle {

LevelViewport::LevelViewport(QWidget* parent)
    : QRhiWidget(parent)
    , m_renderer(std::make_unique<MineRenderer>())
{
    // Set a reasonable default size
    setMinimumSize(640, 480);
}

LevelViewport::~LevelViewport() = default;

void LevelViewport::setMine(const Mine* mine) {
    if (m_renderer) {
        m_renderer->setMine(mine);
        update(); // Request a repaint
    }
}

void LevelViewport::setWireframeMode(bool enabled) {
    if (m_renderer) {
        m_renderer->setWireframeMode(enabled);
        update();
    }
}

bool LevelViewport::isWireframeMode() const {
    return m_renderer ? m_renderer->isWireframeMode() : true;
}

void LevelViewport::initialize(QRhiCommandBuffer* cb)
{
    // Initialize renderer with RHI
    if (m_renderer) {
        m_renderer->initialize(rhi());
    }
    
    // Set up camera
    updateCamera();
    
    Q_UNUSED(cb);
}

void LevelViewport::render(QRhiCommandBuffer* cb)
{
    // Get the render target (the widget's backing texture)
    QRhiRenderTarget* rt = renderTarget();
    if (!rt) {
        return;
    }

    // Begin rendering pass with clear color
    const QColor clearColor = m_clearColor;
    cb->beginPass(rt, clearColor, { 1.0f, 0 });

    // Render mine geometry
    if (m_renderer) {
        m_renderer->render(cb, rt);
    }

    // End the render pass
    cb->endPass();
}

void LevelViewport::updateCamera() {
    // Set up a simple camera looking at the origin
    m_viewMatrix.setToIdentity();
    m_viewMatrix.lookAt(
        QVector3D(m_cameraDistance, m_cameraDistance, m_cameraDistance), // Eye position
        QVector3D(0, 0, 0),                                               // Look at origin
        QVector3D(0, 1, 0)                                                // Up vector
    );

    // Set up perspective projection
    const float aspect = static_cast<float>(width()) / static_cast<float>(height());
    m_projectionMatrix.setToIdentity();
    m_projectionMatrix.perspective(45.0f, aspect, 0.1f, 1000.0f);

    // Update renderer matrices
    if (m_renderer) {
        m_renderer->setViewMatrix(m_viewMatrix);
        m_renderer->setProjectionMatrix(m_projectionMatrix);
    }
}

} // namespace dle
