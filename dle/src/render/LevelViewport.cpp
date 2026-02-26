#include "render/LevelViewport.h"
#include <rhi/qrhi.h>

namespace dle {

LevelViewport::LevelViewport(QWidget* parent)
    : QRhiWidget(parent)
{
    // Set a reasonable default size
    setMinimumSize(640, 480);
}

void LevelViewport::initialize(QRhiCommandBuffer* cb)
{
    // Initialize RHI resources here
    // For now, we just need to set up for clearing
    // Future: create vertex buffers, shaders, pipelines, etc.
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

    // Future rendering will go here:
    // - Set pipeline state
    // - Bind vertex/index buffers
    // - Draw mine geometry
    // - Handle camera transforms

    // End the render pass
    cb->endPass();
}

} // namespace dle
