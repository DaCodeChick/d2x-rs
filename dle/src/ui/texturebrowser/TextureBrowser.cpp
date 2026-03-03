#include "TextureBrowser.h"
#include "../../core/mine/Mine.h"
#include <QPainter>
#include <QMouseEvent>
#include <QWheelEvent>
#include <QScrollBar>
#include <format>

namespace dle {

TextureBrowser::TextureBrowser(QWidget* parent)
    : QWidget(parent)
    , m_mine(nullptr)
    , m_selectedBaseTexture(0)
    , m_selectedOverlayTexture(0)
    , m_textureSize(64)
    , m_spacing(6)
    , m_columns(1)
    , m_scrollOffset(0)
    , m_textureCount(MAX_TEXTURES)
{
    setMinimumWidth(150);
    setMouseTracking(true);
    calculateLayout();
}

TextureBrowser::~TextureBrowser() = default;

void TextureBrowser::setMine(const Mine* mine) {
    m_mine = mine;
    update();
}

void TextureBrowser::setSelectedTextures(int16_t baseTexture, int16_t overlayTexture) {
    if (m_selectedBaseTexture != baseTexture || m_selectedOverlayTexture != overlayTexture) {
        m_selectedBaseTexture = baseTexture;
        m_selectedOverlayTexture = overlayTexture;
        update();
    }
}

void TextureBrowser::calculateLayout() {
    int availableWidth = width() - 20; // Leave margin
    if (availableWidth < m_textureSize + m_spacing) {
        m_columns = 1;
    } else {
        m_columns = availableWidth / (m_textureSize + m_spacing);
        if (m_columns < 1) m_columns = 1;
    }
}

void TextureBrowser::resizeEvent(QResizeEvent* event) {
    QWidget::resizeEvent(event);
    calculateLayout();
}

QRect TextureBrowser::textureRect(int textureId) const {
    if (textureId < 0 || textureId >= m_textureCount) {
        return QRect();
    }
    
    int row = textureId / m_columns;
    int col = textureId % m_columns;
    
    int x = 10 + col * (m_textureSize + m_spacing);
    int y = 10 + row * (m_textureSize + m_spacing) - m_scrollOffset;
    
    return QRect(x, y, m_textureSize, m_textureSize);
}

int16_t TextureBrowser::textureAtPosition(const QPoint& pos) const {
    for (int i = 0; i < m_textureCount; ++i) {
        QRect rect = textureRect(i);
        if (rect.contains(pos)) {
            return static_cast<int16_t>(i);
        }
    }
    return -1;
}

void TextureBrowser::paintEvent(QPaintEvent* /*event*/) {
    QPainter painter(this);
    painter.fillRect(rect(), QColor(40, 40, 40)); // Dark background
    
    // Calculate which textures are visible
    int startRow = m_scrollOffset / (m_textureSize + m_spacing);
    int endRow = (m_scrollOffset + height()) / (m_textureSize + m_spacing) + 1;
    
    int startTexture = startRow * m_columns;
    int endTexture = (endRow + 1) * m_columns;
    
    if (startTexture < 0) startTexture = 0;
    if (endTexture > m_textureCount) endTexture = m_textureCount;
    
    // Draw visible textures
    for (int i = startTexture; i < endTexture; ++i) {
        QRect rect = textureRect(i);
        
        // Skip if not visible
        if (rect.y() + rect.height() < 0 || rect.y() > height()) {
            continue;
        }
        
        // Draw texture placeholder (dark gray square)
        painter.fillRect(rect, QColor(60, 60, 60));
        
        // Draw border
        QColor borderColor = QColor(100, 100, 100);
        if (i == m_selectedBaseTexture) {
            borderColor = QColor(0, 255, 255); // Cyan for base texture
        } else if (i == m_selectedOverlayTexture) {
            borderColor = QColor(255, 255, 0); // Yellow for overlay texture
        }
        painter.setPen(QPen(borderColor, 2));
        painter.drawRect(rect);
        
        // Draw texture ID as text
        painter.setPen(QColor(200, 200, 200));
        painter.drawText(rect, Qt::AlignCenter, QString::number(i));
    }
    
    // Draw info text at top
    painter.fillRect(0, 0, width(), 30, QColor(30, 30, 30));
    painter.setPen(QColor(200, 200, 200));
    painter.drawText(10, 20, std::format("Base: {} | Overlay: {}", 
        m_selectedBaseTexture, m_selectedOverlayTexture).c_str());
}

void TextureBrowser::mousePressEvent(QMouseEvent* event) {
    int16_t textureId = textureAtPosition(event->pos());
    
    if (textureId >= 0) {
        if (event->button() == Qt::LeftButton) {
            // Left click selects base texture
            m_selectedBaseTexture = textureId;
            emit baseTextureSelected(textureId);
            update();
        } else if (event->button() == Qt::RightButton) {
            // Right click selects overlay texture
            m_selectedOverlayTexture = textureId;
            emit overlayTextureSelected(textureId);
            update();
        }
    }
    
    QWidget::mousePressEvent(event);
}

void TextureBrowser::wheelEvent(QWheelEvent* event) {
    // Scroll with mouse wheel
    int delta = -event->angleDelta().y() / 8; // Convert to pixels
    m_scrollOffset += delta;
    
    // Calculate max scroll
    int rows = (m_textureCount + m_columns - 1) / m_columns;
    int totalHeight = rows * (m_textureSize + m_spacing) + 20;
    int maxScroll = totalHeight - height();
    if (maxScroll < 0) maxScroll = 0;
    
    // Clamp scroll offset
    if (m_scrollOffset < 0) m_scrollOffset = 0;
    if (m_scrollOffset > maxScroll) m_scrollOffset = maxScroll;
    
    update();
    event->accept();
}

} // namespace dle
