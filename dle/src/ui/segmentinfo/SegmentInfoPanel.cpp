#include "SegmentInfoPanel.h"
#include "core/mine/Mine.h"
#include "core/mine/Segment.h"
#include <QVBoxLayout>
#include <QHBoxLayout>
#include <QGridLayout>
#include <QGroupBox>
#include <QLabel>
#include <QScrollArea>
#include <format>

namespace dle {

SegmentInfoPanel::SegmentInfoPanel(QWidget* parent)
    : QWidget(parent)
    , m_mine(nullptr)
    , m_selectedSegmentId(-1)
    , m_labelSegmentId(nullptr)
    , m_labelFunction(nullptr)
    , m_labelProperties(nullptr)
    , m_labelStaticLight(nullptr)
    , m_labelCenter(nullptr)
    , m_labelProducer(nullptr)
    , m_labelDamage(nullptr)
{
    for (int i = 0; i < 8; ++i) {
        m_labelVertices[i] = nullptr;
    }
    for (int i = 0; i < 6; ++i) {
        m_labelConnections[i] = nullptr;
    }
    
    setupUi();
}

void SegmentInfoPanel::setMine(const Mine* mine) {
    m_mine = mine;
    m_selectedSegmentId = -1;
    updateDisplay();
}

void SegmentInfoPanel::setSelectedSegment(int16_t segmentId) {
    m_selectedSegmentId = segmentId;
    updateDisplay();
}

void SegmentInfoPanel::refresh() {
    updateDisplay();
}

void SegmentInfoPanel::setupUi() {
    auto* scrollArea = new QScrollArea(this);
    scrollArea->setWidgetResizable(true);
    scrollArea->setFrameShape(QFrame::NoFrame);
    
    auto* scrollWidget = new QWidget();
    auto* mainLayout = new QVBoxLayout(scrollWidget);
    mainLayout->setContentsMargins(8, 8, 8, 8);
    mainLayout->setSpacing(8);
    
    // Basic info group
    auto* basicGroup = new QGroupBox("Segment Info");
    auto* basicLayout = new QGridLayout(basicGroup);
    basicLayout->setColumnStretch(1, 1);
    
    int row = 0;
    basicLayout->addWidget(new QLabel("Segment ID:"), row, 0);
    m_labelSegmentId = new QLabel("(none)");
    basicLayout->addWidget(m_labelSegmentId, row++, 1);
    
    basicLayout->addWidget(new QLabel("Function:"), row, 0);
    m_labelFunction = new QLabel("-");
    basicLayout->addWidget(m_labelFunction, row++, 1);
    
    basicLayout->addWidget(new QLabel("Properties:"), row, 0);
    m_labelProperties = new QLabel("-");
    m_labelProperties->setWordWrap(true);
    basicLayout->addWidget(m_labelProperties, row++, 1);
    
    basicLayout->addWidget(new QLabel("Static Light:"), row, 0);
    m_labelStaticLight = new QLabel("-");
    basicLayout->addWidget(m_labelStaticLight, row++, 1);
    
    basicLayout->addWidget(new QLabel("Center:"), row, 0);
    m_labelCenter = new QLabel("-");
    basicLayout->addWidget(m_labelCenter, row++, 1);
    
    basicLayout->addWidget(new QLabel("Producer:"), row, 0);
    m_labelProducer = new QLabel("-");
    basicLayout->addWidget(m_labelProducer, row++, 1);
    
    basicLayout->addWidget(new QLabel("Damage:"), row, 0);
    m_labelDamage = new QLabel("-");
    basicLayout->addWidget(m_labelDamage, row++, 1);
    
    mainLayout->addWidget(basicGroup);
    
    // Vertices group
    auto* verticesGroup = new QGroupBox("Vertices");
    auto* verticesLayout = new QGridLayout(verticesGroup);
    verticesLayout->setColumnStretch(1, 1);
    
    for (int i = 0; i < 8; ++i) {
        verticesLayout->addWidget(new QLabel(QString("V%1:").arg(i)), i, 0);
        m_labelVertices[i] = new QLabel("-");
        m_labelVertices[i]->setFont(QFont("Monospace", 9));
        verticesLayout->addWidget(m_labelVertices[i], i, 1);
    }
    
    mainLayout->addWidget(verticesGroup);
    
    // Connections group
    auto* connectionsGroup = new QGroupBox("Side Connections");
    auto* connectionsLayout = new QGridLayout(connectionsGroup);
    connectionsLayout->setColumnStretch(1, 1);
    
    const char* sideNames[] = {"Right", "Top", "Front", "Left", "Bottom", "Back"};
    for (int i = 0; i < 6; ++i) {
        connectionsLayout->addWidget(new QLabel(QString("%1:").arg(sideNames[i])), i, 0);
        m_labelConnections[i] = new QLabel("-");
        connectionsLayout->addWidget(m_labelConnections[i], i, 1);
    }
    
    mainLayout->addWidget(connectionsGroup);
    
    mainLayout->addStretch();
    
    scrollArea->setWidget(scrollWidget);
    
    auto* topLayout = new QVBoxLayout(this);
    topLayout->setContentsMargins(0, 0, 0, 0);
    topLayout->addWidget(scrollArea);
    
    updateDisplay();
}

void SegmentInfoPanel::updateDisplay() {
    if (!m_mine || m_selectedSegmentId < 0 || m_selectedSegmentId >= m_mine->getSegmentCount()) {
        // No valid selection
        m_labelSegmentId->setText("(none)");
        m_labelFunction->setText("-");
        m_labelProperties->setText("-");
        m_labelStaticLight->setText("-");
        m_labelCenter->setText("-");
        m_labelProducer->setText("-");
        m_labelDamage->setText("-");
        
        for (int i = 0; i < 8; ++i) {
            m_labelVertices[i]->setText("-");
        }
        for (int i = 0; i < 6; ++i) {
            m_labelConnections[i]->setText("-");
        }
        return;
    }
    
    const auto& segment = m_mine->getSegment(m_selectedSegmentId);
    
    // Basic info
    m_labelSegmentId->setText(QString::number(m_selectedSegmentId));
    m_labelFunction->setText(formatFunction(static_cast<int>(segment.getFunction())));
    m_labelProperties->setText(formatProperties(segment.getProperties()));
    m_labelStaticLight->setText(QString::number(segment.getStaticLight()));
    
    // Center coordinates
    const auto& center = segment.getCenter();
    m_labelCenter->setText(QString("(%1, %2, %3)")
        .arg(center.x, 0, 'f', 2)
        .arg(center.y, 0, 'f', 2)
        .arg(center.z, 0, 'f', 2));
    
    // Producer
    int16_t producerId = segment.getProducerId();
    if (producerId >= 0) {
        m_labelProducer->setText(QString("ID %1, Value %2").arg(producerId).arg(segment.getValue()));
    } else {
        m_labelProducer->setText("None");
    }
    
    // Damage
    m_labelDamage->setText(QString("Shields: %1, Energy: %2")
        .arg(segment.getDamage(0))
        .arg(segment.getDamage(1)));
    
    // Vertices
    for (int i = 0; i < 8; ++i) {
        uint16_t vertexId = segment.getVertexId(i);
        if (vertexId < m_mine->getVertexCount()) {
            const auto& vertex = m_mine->getVertex(vertexId);
            const auto& pos = vertex.position;
            m_labelVertices[i]->setText(QString("#%1 (%2, %3, %4)")
                .arg(vertexId, 4)
                .arg(fixToDouble(pos.x), 7, 'f', 1)
                .arg(fixToDouble(pos.y), 7, 'f', 1)
                .arg(fixToDouble(pos.z), 7, 'f', 1));
        } else {
            m_labelVertices[i]->setText(QString("#%1 (invalid)").arg(vertexId));
        }
    }
    
    // Connections
    for (int i = 0; i < 6; ++i) {
        int16_t childId = segment.getChildId(i);
        m_labelConnections[i]->setText(formatConnection(childId));
    }
}

QString SegmentInfoPanel::formatFunction(int funcValue) const {
    switch (static_cast<SegmentFunction>(funcValue)) {
        case SegmentFunction::NONE: return "None";
        case SegmentFunction::ENERGY_CENTER: return "Energy Center";
        case SegmentFunction::REPAIR_CENTER: return "Repair Center";
        case SegmentFunction::REACTOR: return "Reactor";
        case SegmentFunction::ROBOT_MAKER: return "Robot Matcen";
        case SegmentFunction::GOAL_BLUE: return "Blue Goal";
        case SegmentFunction::GOAL_RED: return "Red Goal";
        case SegmentFunction::TEAM_BLUE: return "Blue Team Area";
        case SegmentFunction::TEAM_RED: return "Red Team Area";
        case SegmentFunction::SPEED_BOOST: return "Speed Boost";
        case SegmentFunction::SKYBOX: return "Skybox";
        case SegmentFunction::EQUIP_MAKER: return "Equipment Maker";
        default: return QString("Unknown (%1)").arg(funcValue);
    }
}

QString SegmentInfoPanel::formatProperties(uint8_t props) const {
    if (props == 0) {
        return "None";
    }
    
    QStringList propList;
    if (props & static_cast<uint8_t>(SegmentProperty::WATER)) propList << "Water";
    if (props & static_cast<uint8_t>(SegmentProperty::LAVA)) propList << "Lava";
    if (props & static_cast<uint8_t>(SegmentProperty::BLOCKED)) propList << "Blocked";
    if (props & static_cast<uint8_t>(SegmentProperty::NO_DAMAGE)) propList << "No Damage";
    if (props & static_cast<uint8_t>(SegmentProperty::SELF_ILLUMINATE)) propList << "Self-Illuminate";
    if (props & static_cast<uint8_t>(SegmentProperty::LIGHT_FOG)) propList << "Light Fog";
    if (props & static_cast<uint8_t>(SegmentProperty::DENSE_FOG)) propList << "Dense Fog";
    
    return propList.join(", ");
}

QString SegmentInfoPanel::formatConnection(int16_t childId) const {
    if (childId < 0) {
        return "(no connection)";
    }
    return QString("Segment #%1").arg(childId);
}

} // namespace dle
