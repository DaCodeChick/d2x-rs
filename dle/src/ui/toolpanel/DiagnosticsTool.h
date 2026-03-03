#pragma once

#include <QWidget>
#include <memory>

namespace Ui {
class DiagnosticsTool;
}

namespace dle {

class Mine;

/**
 * @brief Diagnostics tool panel
 * 
 * Provides level statistics and validation:
 * - Level statistics (segments, vertices, walls, triggers, objects)
 * - Object type counts (robots, hostages, players, powerups, reactors)
 * - Mine validation ("Check Mine" button)
 * - Issues list display
 */
class DiagnosticsTool : public QWidget {
    Q_OBJECT

public:
    explicit DiagnosticsTool(QWidget* parent = nullptr);
    ~DiagnosticsTool() override;

    /**
     * @brief Set the mine data to observe
     * @param mine Pointer to mine (non-owning, can be nullptr)
     */
    void setMine(const Mine* mine);

    /**
     * @brief Update UI from current mine data
     */
    void refresh();

private slots:
    void onCheckMineClicked();
    void onRefreshClicked();

private:
    /**
     * @brief Update statistics display
     */
    void updateStatistics();
    
    /**
     * @brief Count objects by type
     */
    void countObjectTypes(int& robots, int& hostages, int& players, int& powerups, int& reactors) const;

private:
    std::unique_ptr<Ui::DiagnosticsTool> ui;
    const Mine* m_mine;  ///< Non-owning observer pointer
};

} // namespace dle
