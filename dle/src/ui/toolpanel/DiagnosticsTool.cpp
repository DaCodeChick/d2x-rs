#include "DiagnosticsTool.h"
#include "ui_DiagnosticsTool.h"
#include "../../core/mine/Mine.h"
#include "../../core/mine/Object.h"
#include <format>

namespace dle {

DiagnosticsTool::DiagnosticsTool(QWidget* parent)
    : QWidget(parent)
    , ui(std::make_unique<Ui::DiagnosticsTool>())
    , m_mine(nullptr)
{
    ui->setupUi(this);
    
    // Connect signals
    connect(ui->btnCheckMine, &QPushButton::clicked, this, &DiagnosticsTool::onCheckMineClicked);
    connect(ui->btnRefresh, &QPushButton::clicked, this, &DiagnosticsTool::onRefreshClicked);
}

DiagnosticsTool::~DiagnosticsTool() = default;

void DiagnosticsTool::setMine(const Mine* mine) {
    m_mine = mine;
    refresh();
}

void DiagnosticsTool::refresh() {
    if (!m_mine) {
        return;
    }
    
    updateStatistics();
}

void DiagnosticsTool::onCheckMineClicked() {
    // TODO: Implement mine validation
    // This would check for:
    // - Invalid segment connections
    // - Missing required objects (player starts, reactor, etc.)
    // - Wall/trigger consistency
    // - Vertex precision issues
    // - etc.
    
    ui->issuesList->clear();
    ui->issuesList->addItem("Mine validation not yet implemented");
}

void DiagnosticsTool::onRefreshClicked() {
    refresh();
}

void DiagnosticsTool::updateStatistics() {
    if (!m_mine) {
        return;
    }
    
    // Update basic statistics
    ui->valueSegments->setText(QString::number(m_mine->getSegmentCount()));
    ui->valueVertices->setText(QString::number(m_mine->getVertexCount()));
    ui->valueWalls->setText(QString::number(m_mine->getWallCount()));
    ui->valueTriggers->setText(QString::number(m_mine->getTriggerCount()));
    ui->valueObjects->setText(QString::number(m_mine->getObjectCount()));
    
    // Count objects by type
    int robots = 0, hostages = 0, players = 0, powerups = 0, reactors = 0;
    countObjectTypes(robots, hostages, players, powerups, reactors);
    
    ui->valueRobots->setText(QString::number(robots));
    ui->valueHostages->setText(QString::number(hostages));
    ui->valuePlayers->setText(QString::number(players));
    ui->valuePowerups->setText(QString::number(powerups));
    ui->valueReactors->setText(QString::number(reactors));
}

void DiagnosticsTool::countObjectTypes(int& robots, int& hostages, int& players, int& powerups, int& reactors) const {
    robots = 0;
    hostages = 0;
    players = 0;
    powerups = 0;
    reactors = 0;
    
    if (!m_mine) {
        return;
    }
    
    for (int i = 0; i < m_mine->getObjectCount(); ++i) {
        const Object& obj = m_mine->getObject(i);
        
        switch (obj.getType()) {
            case ObjectType::Robot:
                robots++;
                break;
            case ObjectType::Hostage:
                hostages++;
                break;
            case ObjectType::Player:
            case ObjectType::Coop:
                players++;
                break;
            case ObjectType::Powerup:
                powerups++;
                break;
            case ObjectType::Reactor:
                reactors++;
                break;
            default:
                break;
        }
    }
}

} // namespace dle
