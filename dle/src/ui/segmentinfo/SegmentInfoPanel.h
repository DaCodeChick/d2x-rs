#pragma once

#include <QWidget>
#include <QLabel>
#include <cstdint>

namespace dle {

class Mine;
class Segment;

/**
 * @brief Panel showing detailed information about the currently selected segment
 * 
 * Displays:
 * - Segment ID and function
 * - 8 vertex IDs with coordinates
 * - 6 side connections (children)
 * - Properties (water, lava, fog, etc.)
 * - Static light level
 * - Matcen/producer info
 * - Damage values
 * - Center point coordinates
 */
class SegmentInfoPanel : public QWidget {
    Q_OBJECT

public:
    explicit SegmentInfoPanel(QWidget* parent = nullptr);
    ~SegmentInfoPanel() override = default;

    /**
     * @brief Set the mine to observe for segment data
     * @param mine Pointer to mine (non-owning observation)
     */
    void setMine(const Mine* mine);

    /**
     * @brief Set the currently selected segment to display
     * @param segmentId ID of segment to show info for (-1 for none)
     */
    void setSelectedSegment(int16_t segmentId);

public slots:
    /**
     * @brief Update the panel to show current segment info
     */
    void refresh();

private:
    void setupUi();
    void updateDisplay();
    QString formatFunction(int funcValue) const;
    QString formatProperties(uint8_t props) const;
    QString formatConnection(int16_t childId) const;

    const Mine* m_mine;
    int16_t m_selectedSegmentId;

    // UI will be laid out in code (no .ui file for this simple panel)
    QLabel* m_labelSegmentId;
    QLabel* m_labelFunction;
    QLabel* m_labelProperties;
    QLabel* m_labelStaticLight;
    QLabel* m_labelCenter;
    QLabel* m_labelProducer;
    QLabel* m_labelDamage;
    
    // Vertex info (8 rows)
    QLabel* m_labelVertices[8];
    
    // Side connections (6 rows)
    QLabel* m_labelConnections[6];
};

} // namespace dle
