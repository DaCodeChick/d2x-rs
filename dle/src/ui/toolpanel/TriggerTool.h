#pragma once

#include <QWidget>
#include <memory>

namespace Ui {
class TriggerTool;
}

namespace dle {

class Mine;
class Trigger;

/**
 * @brief Trigger editing tool panel
 * 
 * Provides controls for editing trigger properties including:
 * - Trigger type (open door, matcen, exit, etc.)
 * - Value and time settings
 * - Trigger flags (one shot, disabled, etc.)
 * - Target list management
 * - Basic operations (add, delete, navigate)
 */
class TriggerTool : public QWidget {
    Q_OBJECT

public:
    explicit TriggerTool(QWidget *parent = nullptr);
    ~TriggerTool();

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
     * @brief Emitted when a trigger property is modified
     */
    void triggerModified(int triggerId);

private slots:
    void onTriggerIdChanged(int value);
    void onTypeChanged(int index);
    void onValueChanged(double value);
    void onTimeChanged(double value);
    void onFlagToggled(bool checked);
    void onAddTarget();
    void onRemoveTarget();
    void onAddTrigger();
    void onDeleteTrigger();

private:
    void setupConnections();
    void updateDisplay();
    void updateTargetsList();
    void enableControls(bool enable);
    
    std::unique_ptr<Ui::TriggerTool> ui;
    const Mine* m_mine;
    int m_currentTriggerId;
};

} // namespace dle
