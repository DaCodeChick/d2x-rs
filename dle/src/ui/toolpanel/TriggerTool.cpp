#include "TriggerTool.h"
#include "ui_TriggerTool.h"
#include "../../core/mine/Mine.h"
#include "../../core/mine/Trigger.h"
#include <QSignalBlocker>

namespace dle {

TriggerTool::TriggerTool(QWidget *parent)
    : QWidget(parent)
    , ui(std::make_unique<Ui::TriggerTool>())
    , m_mine(nullptr)
    , m_currentTriggerId(0)
{
    ui->setupUi(this);
    setupConnections();
    enableControls(false);
}

TriggerTool::~TriggerTool() = default;

void TriggerTool::setMine(const Mine* mine) {
    m_mine = mine;
    
    if (m_mine && m_mine->getTriggerCount() > 0) {
        m_currentTriggerId = 0;
        ui->triggerIdSpin->setMaximum(m_mine->getTriggerCount() - 1);
        enableControls(true);
        refresh();
    } else {
        m_currentTriggerId = 0;
        enableControls(false);
    }
}

void TriggerTool::refresh() {
    if (!m_mine || m_currentTriggerId < 0 || m_currentTriggerId >= m_mine->getTriggerCount()) {
        enableControls(false);
        return;
    }
    
    updateDisplay();
}

void TriggerTool::setupConnections() {
    connect(ui->triggerIdSpin, QOverload<int>::of(&QSpinBox::valueChanged),
            this, &TriggerTool::onTriggerIdChanged);
    connect(ui->typeCombo, QOverload<int>::of(&QComboBox::currentIndexChanged),
            this, &TriggerTool::onTypeChanged);
    connect(ui->valueSpin, QOverload<double>::of(&QDoubleSpinBox::valueChanged),
            this, &TriggerTool::onValueChanged);
    connect(ui->timeSpin, QOverload<double>::of(&QDoubleSpinBox::valueChanged),
            this, &TriggerTool::onTimeChanged);
    
    connect(ui->noMessageCheck, &QCheckBox::toggled, this, &TriggerTool::onFlagToggled);
    connect(ui->oneShotCheck, &QCheckBox::toggled, this, &TriggerTool::onFlagToggled);
    connect(ui->disabledCheck, &QCheckBox::toggled, this, &TriggerTool::onFlagToggled);
    connect(ui->onCheck, &QCheckBox::toggled, this, &TriggerTool::onFlagToggled);
    connect(ui->permanentCheck, &QCheckBox::toggled, this, &TriggerTool::onFlagToggled);
    connect(ui->alternateCheck, &QCheckBox::toggled, this, &TriggerTool::onFlagToggled);
    
    connect(ui->addTargetBtn, &QPushButton::clicked, this, &TriggerTool::onAddTarget);
    connect(ui->removeTargetBtn, &QPushButton::clicked, this, &TriggerTool::onRemoveTarget);
    connect(ui->addTriggerBtn, &QPushButton::clicked, this, &TriggerTool::onAddTrigger);
    connect(ui->deleteTriggerBtn, &QPushButton::clicked, this, &TriggerTool::onDeleteTrigger);
}

void TriggerTool::updateDisplay() {
    if (!m_mine) return;
    
    const Trigger& trigger = m_mine->getTrigger(m_currentTriggerId);
    
    QSignalBlocker b1(ui->triggerIdSpin), b2(ui->typeCombo), b3(ui->valueSpin), b4(ui->timeSpin);
    QSignalBlocker b5(ui->noMessageCheck), b6(ui->oneShotCheck), b7(ui->disabledCheck);
    QSignalBlocker b8(ui->onCheck), b9(ui->permanentCheck), b10(ui->alternateCheck);
    
    ui->triggerIdSpin->setValue(m_currentTriggerId);
    ui->typeCombo->setCurrentIndex(static_cast<int>(trigger.getType()));
    ui->valueSpin->setValue(static_cast<double>(trigger.getValue()) / 65536.0);
    ui->timeSpin->setValue(static_cast<double>(trigger.getTime()) / 65536.0);
    
    ui->noMessageCheck->setChecked(trigger.hasFlag(TriggerNoMessage));
    ui->oneShotCheck->setChecked(trigger.hasFlag(TriggerOneShot));
    ui->disabledCheck->setChecked(trigger.hasFlag(TriggerDisabled));
    ui->onCheck->setChecked(trigger.hasFlag(TriggerOn));
    ui->permanentCheck->setChecked(trigger.hasFlag(TriggerPermanent));
    ui->alternateCheck->setChecked(trigger.hasFlag(TriggerAlternate));
    
    updateTargetsList();
}

void TriggerTool::updateTargetsList() {
    if (!m_mine) return;
    
    const Trigger& trigger = m_mine->getTrigger(m_currentTriggerId);
    ui->targetsList->clear();
    
    for (int i = 0; i < trigger.getTargetCount(); ++i) {
        const auto& target = trigger.getTarget(i);
        ui->targetsList->addItem(QString("Seg:%1 Side:%2")
            .arg(target.segmentId).arg(target.sideId));
    }
}

void TriggerTool::enableControls(bool enable) {
    ui->triggerDataGroup->setEnabled(enable);
    ui->flagsGroup->setEnabled(enable);
    ui->targetsGroup->setEnabled(enable);
    ui->actionsGroup->setEnabled(enable);
}

void TriggerTool::onTriggerIdChanged(int value) {
    if (!m_mine || value < 0 || value >= m_mine->getTriggerCount()) return;
    m_currentTriggerId = value;
    updateDisplay();
}

void TriggerTool::onTypeChanged(int index) {
    if (!m_mine) return;
    auto* mutableMine = const_cast<Mine*>(m_mine);
    mutableMine->getTrigger(m_currentTriggerId).setType(static_cast<TriggerType>(index));
    emit triggerModified(m_currentTriggerId);
}

void TriggerTool::onValueChanged(double value) {
    if (!m_mine) return;
    auto* mutableMine = const_cast<Mine*>(m_mine);
    mutableMine->getTrigger(m_currentTriggerId).setValue(static_cast<fix>(value * 65536.0));
    emit triggerModified(m_currentTriggerId);
}

void TriggerTool::onTimeChanged(double value) {
    if (!m_mine) return;
    auto* mutableMine = const_cast<Mine*>(m_mine);
    mutableMine->getTrigger(m_currentTriggerId).setTime(static_cast<fix>(value * 65536.0));
    emit triggerModified(m_currentTriggerId);
}

void TriggerTool::onFlagToggled(bool checked) {
    if (!m_mine) return;
    
    auto* sender = qobject_cast<QCheckBox*>(QObject::sender());
    if (!sender) return;
    
    auto* mutableMine = const_cast<Mine*>(m_mine);
    Trigger& trigger = mutableMine->getTrigger(m_currentTriggerId);
    
    TriggerFlags flag = TriggerNoMessage;
    if (sender == ui->noMessageCheck) flag = TriggerNoMessage;
    else if (sender == ui->oneShotCheck) flag = TriggerOneShot;
    else if (sender == ui->disabledCheck) flag = TriggerDisabled;
    else if (sender == ui->onCheck) flag = TriggerOn;
    else if (sender == ui->permanentCheck) flag = TriggerPermanent;
    else if (sender == ui->alternateCheck) flag = TriggerAlternate;
    
    if (checked) trigger.addFlag(flag);
    else trigger.removeFlag(flag);
    
    emit triggerModified(m_currentTriggerId);
}

void TriggerTool::onAddTarget() {
    // TODO: Implement add target (requires segment/side selection dialog)
}

void TriggerTool::onRemoveTarget() {
    // TODO: Implement remove target
}

void TriggerTool::onAddTrigger() {
    // TODO: Implement add trigger
}

void TriggerTool::onDeleteTrigger() {
    // TODO: Implement delete trigger
}

} // namespace dle
