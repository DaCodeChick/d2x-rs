#pragma once

#include <QWidget>
#include <memory>

namespace Ui {
class WallTool;
}

namespace dle {

class Mine;
class Wall;

/**
 * @brief Wall editing tool panel
 * 
 * Provides controls for editing wall/door properties including:
 * - Wall type (normal, door, illusion, etc.)
 * - Animation clip number
 * - Strength and cloak values
 * - Key requirements
 * - Wall flags (blasted, locked, auto, etc.)
 * - Basic operations (add, delete, navigate)
 */
class WallTool : public QWidget {
    Q_OBJECT

public:
    explicit WallTool(QWidget *parent = nullptr);
    ~WallTool();

    /**
     * @brief Set the mine data source (non-owning observer)
     */
    void setMine(const Mine* mine);

    /**
     * @brief Refresh the tool with current mine data
     */
    void refresh();

signals:
    /**
     * @brief Emitted when a wall property is modified
     */
    void wallModified(int wallId);

private slots:
    void onWallIdChanged(int value);
    void onTypeChanged(int index);
    void onClipChanged(int value);
    void onStrengthChanged(double value);
    void onCloakChanged(int value);
    void onKeyToggled(bool checked);
    void onFlagToggled(bool checked);
    void onAddWall();
    void onDeleteWall();
    void onOtherSide();

private:
    void setupConnections();
    void updateDisplay();
    void enableControls(bool enable);
    
    std::unique_ptr<Ui::WallTool> ui;
    const Mine* m_mine;
    int m_currentWallId;
};

} // namespace dle
