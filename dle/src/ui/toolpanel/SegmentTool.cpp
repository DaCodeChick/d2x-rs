#include "SegmentTool.h"
#include "ui_SegmentTool.h"
#include "../../core/mine/Mine.h"
#include "../../core/mine/Segment.h"
#include <QSignalBlocker>

namespace dle {

SegmentTool::SegmentTool(QWidget *parent)
    : QWidget(parent)
    , ui(std::make_unique<Ui::SegmentTool>())
    , m_mine(nullptr)
    , m_currentSegmentId(0)
{
    ui->setupUi(this);
    setupConnections();
    enableControls(false);
}

SegmentTool::~SegmentTool() = default;

void SegmentTool::setMine(const Mine* mine) {
    m_mine = mine;
    
    if (m_mine && m_mine->getSegmentCount() > 0) {
        m_currentSegmentId = 0;
        ui->segmentIdSpin->setMaximum(m_mine->getSegmentCount() - 1);
        enableControls(true);
        refresh();
    } else {
        m_currentSegmentId = 0;
        enableControls(false);
    }
}

void SegmentTool::refresh() {
    if (!m_mine || m_currentSegmentId < 0 || m_currentSegmentId >= m_mine->getSegmentCount()) {
        enableControls(false);
        return;
    }
    
    updateDisplay();
}

void SegmentTool::setupConnections() {
    // Segment ID navigation
    connect(ui->segmentIdSpin, QOverload<int>::of(&QSpinBox::valueChanged),
            this, &SegmentTool::onSegmentIdChanged);
    
    // Function and lighting
    connect(ui->functionCombo, QOverload<int>::of(&QComboBox::currentIndexChanged),
            this, &SegmentTool::onFunctionChanged);
    connect(ui->lightSpin, QOverload<int>::of(&QSpinBox::valueChanged),
            this, &SegmentTool::onLightChanged);
    
    // Property checkboxes
    connect(ui->waterCheck, &QCheckBox::toggled, this, &SegmentTool::onPropertyToggled);
    connect(ui->lavaCheck, &QCheckBox::toggled, this, &SegmentTool::onPropertyToggled);
    connect(ui->blockedCheck, &QCheckBox::toggled, this, &SegmentTool::onPropertyToggled);
    connect(ui->noDamageCheck, &QCheckBox::toggled, this, &SegmentTool::onPropertyToggled);
    connect(ui->selfIlluminateCheck, &QCheckBox::toggled, this, &SegmentTool::onPropertyToggled);
    connect(ui->lightFogCheck, &QCheckBox::toggled, this, &SegmentTool::onPropertyToggled);
    connect(ui->denseFogCheck, &QCheckBox::toggled, this, &SegmentTool::onPropertyToggled);
    
    // Action buttons
    connect(ui->addSegmentBtn, &QPushButton::clicked, this, &SegmentTool::onAddSegment);
    connect(ui->deleteSegmentBtn, &QPushButton::clicked, this, &SegmentTool::onDeleteSegment);
    connect(ui->splitIn7Btn, &QPushButton::clicked, this, &SegmentTool::onSplitSegment7);
    connect(ui->splitIn8Btn, &QPushButton::clicked, this, &SegmentTool::onSplitSegment8);
}

void SegmentTool::updateDisplay() {
    if (!m_mine) {
        return;
    }
    
    const Segment& segment = m_mine->getSegment(m_currentSegmentId);
    
    // Block signals during update to avoid recursion
    {
        QSignalBlocker blocker1(ui->segmentIdSpin);
        QSignalBlocker blocker2(ui->functionCombo);
        QSignalBlocker blocker3(ui->lightSpin);
        QSignalBlocker blocker4(ui->waterCheck);
        QSignalBlocker blocker5(ui->lavaCheck);
        QSignalBlocker blocker6(ui->blockedCheck);
        QSignalBlocker blocker7(ui->noDamageCheck);
        QSignalBlocker blocker8(ui->selfIlluminateCheck);
        QSignalBlocker blocker9(ui->lightFogCheck);
        QSignalBlocker blocker10(ui->denseFogCheck);
        
        // Update segment ID
        ui->segmentIdSpin->setValue(m_currentSegmentId);
        
        // Update function combobox
        // Map SegmentFunction enum to combo box index
        int functionIndex = static_cast<int>(segment.getFunction());
        // UI combo box order: None, Energy Center, Repair Center, Reactor, Robot Maker, Equipment Maker
        // This matches the enum order for indices 0-5, and we map EQUIP_MAKER (11) to index 5
        if (functionIndex == 11) { // EQUIP_MAKER
            functionIndex = 5;
        } else if (functionIndex > 5) {
            functionIndex = 0; // Unsupported functions show as "None"
        }
        ui->functionCombo->setCurrentIndex(functionIndex);
        
        // Update light value (convert from 0-2.0 fixed to 0-200 percentage)
        int lightPercent = segment.getStaticLight() / 327; // 65536 / 200 ≈ 327
        ui->lightSpin->setValue(lightPercent);
        
        // Update property checkboxes
        ui->waterCheck->setChecked(segment.hasProperty(SegmentProperty::WATER));
        ui->lavaCheck->setChecked(segment.hasProperty(SegmentProperty::LAVA));
        ui->blockedCheck->setChecked(segment.hasProperty(SegmentProperty::BLOCKED));
        ui->noDamageCheck->setChecked(segment.hasProperty(SegmentProperty::NO_DAMAGE));
        ui->selfIlluminateCheck->setChecked(segment.hasProperty(SegmentProperty::SELF_ILLUMINATE));
        ui->lightFogCheck->setChecked(segment.hasProperty(SegmentProperty::LIGHT_FOG));
        ui->denseFogCheck->setChecked(segment.hasProperty(SegmentProperty::DENSE_FOG));
    }
}

void SegmentTool::enableControls(bool enable) {
    ui->segmentDataGroup->setEnabled(enable);
    ui->propertiesGroup->setEnabled(enable);
    ui->actionsGroup->setEnabled(enable);
}

void SegmentTool::onSegmentIdChanged(int value) {
    if (!m_mine || value < 0 || value >= m_mine->getSegmentCount()) {
        return;
    }
    
    m_currentSegmentId = value;
    updateDisplay();
}

void SegmentTool::onFunctionChanged(int index) {
    if (!m_mine) {
        return;
    }
    
    // Map combo box index back to SegmentFunction enum
    SegmentFunction func = SegmentFunction::NONE;
    switch (index) {
        case 0: func = SegmentFunction::NONE; break;
        case 1: func = SegmentFunction::ENERGY_CENTER; break;
        case 2: func = SegmentFunction::REPAIR_CENTER; break;
        case 3: func = SegmentFunction::REACTOR; break;
        case 4: func = SegmentFunction::ROBOT_MAKER; break;
        case 5: func = SegmentFunction::EQUIP_MAKER; break;
        default: func = SegmentFunction::NONE; break;
    }
    
    // Note: We're observing a const Mine*, so we can't modify it directly
    // In a real implementation, we would emit a signal to request modification
    // For now, we'll cast away const (this is temporary until we implement proper command pattern)
    auto* mutableMine = const_cast<Mine*>(m_mine);
    mutableMine->getSegment(m_currentSegmentId).setFunction(func);
    
    emit segmentModified(m_currentSegmentId);
}

void SegmentTool::onLightChanged(int value) {
    if (!m_mine) {
        return;
    }
    
    // Convert percentage (0-200) to fixed point (0-131072 where 65536 = 1.0)
    int lightValue = value * 327; // Approximate conversion
    
    auto* mutableMine = const_cast<Mine*>(m_mine);
    mutableMine->getSegment(m_currentSegmentId).setStaticLight(lightValue);
    
    emit segmentModified(m_currentSegmentId);
}

void SegmentTool::onPropertyToggled(bool checked) {
    if (!m_mine) {
        return;
    }
    
    auto* sender = qobject_cast<QCheckBox*>(QObject::sender());
    if (!sender) {
        return;
    }
    
    auto* mutableMine = const_cast<Mine*>(m_mine);
    Segment& segment = mutableMine->getSegment(m_currentSegmentId);
    
    // Determine which property was toggled
    SegmentProperty prop = SegmentProperty::NONE;
    if (sender == ui->waterCheck) {
        prop = SegmentProperty::WATER;
    } else if (sender == ui->lavaCheck) {
        prop = SegmentProperty::LAVA;
    } else if (sender == ui->blockedCheck) {
        prop = SegmentProperty::BLOCKED;
    } else if (sender == ui->noDamageCheck) {
        prop = SegmentProperty::NO_DAMAGE;
    } else if (sender == ui->selfIlluminateCheck) {
        prop = SegmentProperty::SELF_ILLUMINATE;
    } else if (sender == ui->lightFogCheck) {
        prop = SegmentProperty::LIGHT_FOG;
    } else if (sender == ui->denseFogCheck) {
        prop = SegmentProperty::DENSE_FOG;
    }
    
    if (checked) {
        segment.addProperty(prop);
    } else {
        segment.removeProperty(prop);
    }
    
    emit segmentModified(m_currentSegmentId);
}

void SegmentTool::onAddSegment() {
    // TODO: Implement add segment functionality
    // This requires geometry generation and mine structure modification
}

void SegmentTool::onDeleteSegment() {
    // TODO: Implement delete segment functionality
    // This requires careful handling of connectivity and vertex cleanup
}

void SegmentTool::onSplitSegment7() {
    // TODO: Implement split into 7 segments
    // This is a complex geometric operation (6 + 1 center segment)
}

void SegmentTool::onSplitSegment8() {
    // TODO: Implement split into 8 segments
    // This is a complex geometric operation (8 equal octants)
}

} // namespace dle
