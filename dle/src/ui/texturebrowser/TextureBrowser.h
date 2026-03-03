#pragma once

#include <QWidget>
#include <QScrollArea>
#include <vector>

namespace dle {

class Mine;

/**
 * @brief Texture browser widget - displays textures in a scrollable grid
 * 
 * Provides texture selection interface:
 * - Scrollable grid of texture thumbnails
 * - Click to select base texture (left click)
 * - Right-click to select overlay texture
 * - Filter by texture type/category
 * - Display current selection
 * 
 * Note: Currently displays texture IDs as placeholders until
 * texture loading from .PIG files is implemented.
 */
class TextureBrowser : public QWidget {
    Q_OBJECT

public:
    explicit TextureBrowser(QWidget* parent = nullptr);
    ~TextureBrowser() override;

    /**
     * @brief Set the mine data to observe
     * @param mine Pointer to mine (non-owning, can be nullptr)
     */
    void setMine(const Mine* mine);

    /**
     * @brief Get currently selected base texture
     */
    int16_t getSelectedBaseTexture() const { return m_selectedBaseTexture; }

    /**
     * @brief Get currently selected overlay texture
     */
    int16_t getSelectedOverlayTexture() const { return m_selectedOverlayTexture; }

    /**
     * @brief Set selected textures
     */
    void setSelectedTextures(int16_t baseTexture, int16_t overlayTexture);

signals:
    /**
     * @brief Emitted when base texture selection changes
     */
    void baseTextureSelected(int16_t textureId);

    /**
     * @brief Emitted when overlay texture selection changes
     */
    void overlayTextureSelected(int16_t textureId);

protected:
    void paintEvent(QPaintEvent* event) override;
    void mousePressEvent(QMouseEvent* event) override;
    void resizeEvent(QResizeEvent* event) override;
    void wheelEvent(QWheelEvent* event) override;

private:
    /**
     * @brief Calculate layout parameters
     */
    void calculateLayout();

    /**
     * @brief Get texture ID at screen position
     * @return Texture ID or -1 if none
     */
    int16_t textureAtPosition(const QPoint& pos) const;

    /**
     * @brief Get rectangle for texture in grid
     */
    QRect textureRect(int textureId) const;

private:
    const Mine* m_mine;                    ///< Non-owning observer pointer
    
    // Selection state
    int16_t m_selectedBaseTexture;         ///< Currently selected base texture
    int16_t m_selectedOverlayTexture;      ///< Currently selected overlay texture
    
    // Layout parameters
    int m_textureSize;                     ///< Size of each texture thumbnail (pixels)
    int m_spacing;                         ///< Spacing between thumbnails
    int m_columns;                         ///< Number of columns in grid
    int m_scrollOffset;                    ///< Vertical scroll offset
    
    // Texture data
    int m_textureCount;                    ///< Total number of textures
    static constexpr int MAX_TEXTURES = 910; ///< Descent 2 has ~910 textures
};

} // namespace dle
