#include "WallTool.h"
#include "ui_WallTool.h"
#include "../../core/mine/Mine.h"
#include "../../core/mine/Wall.h"
#include <QSignalBlocker>

namespace dle {

WallTool::WallTool(QWidget *parent)
    : QWidget(parent)
    , ui(std::make_unique<Ui::WallTool>())
    , m_mine(nullptr)
    , m_currentWallId(0)
{
    ui->setupUi(this);
    setupConnections();
    enableControls(false);
}

WallTool::~WallTool() = default;

void WallTool::setMine(const Mine* mine) {
    m_mine = mine;
    
    if (m_mine && m_mine->getWallCount() > 0) {
        m_currentWallId = 0;
        ui->wallIdSpin->setMaximum(m_mine->getWallCount() - 1);
        enableControls(true);
        refresh();
    } else {
        m_currentWallId = 0;
        enableControls(false);
    }
}

void WallTool::refresh() {
    if (!m_mine || m_currentWallId < 0 || m_currentWallId >= m_mine->getWallCount()) {
        enableControls(false);
        return;
    }
    
    updateDisplay();
}

void WallTool::setupConnections() {
    // Wall ID navigation
    connect(ui->wallIdSpin, QOverload<int>::of(&QSpinBox::valueChanged),
            this, &WallTool::onWallIdChanged);
    
    // Wall properties
    connect(ui->typeCombo, QOverload<int>::of(&QComboBox::currentIndexChanged),
            this, &WallTool::onTypeChanged);
    connect(ui->clipSpin, QOverload<int>::of(&QSpinBox::valueChanged),
            this, &WallTool::onClipChanged);
    connect(ui->strengthSpin, QOverload<double>::of(&QDoubleSpinBox::valueChanged),
            this, &WallTool::onStrengthChanged);
    connect(ui->cloakSpin, QOverload<int>::of(&QSpinBox::valueChanged),
            this, &WallTool::onCloakChanged);
    
    // Key checkboxes
    connect(ui->noKeyCheck, &QCheckBox::toggled, this, &WallTool::onKeyToggled);
    connect(ui->blueKeyCheck, &QCheckBox::toggled, this, &WallTool::onKeyToggled);
    connect(ui->redKeyCheck, &QCheckBox::toggled, this, &WallTool::onKeyToggled);
    connect(ui->goldKeyCheck, &QCheckBox::toggled, this, &WallTool::onKeyToggled);
    
    // Flag checkboxes
    connect(ui->blastedCheck, &QCheckBox::toggled, this, &WallTool::onFlagToggled);
    connect(ui->doorOpenCheck, &QCheckBox::toggled, this, &WallTool::onFlagToggled);
    connect(ui->doorLockedCheck, &QCheckBox::toggled, this, &WallTool::onFlagToggled);
    connect(ui->doorAutoCheck, &QCheckBox::toggled, this, &WallTool::onFlagToggled);
    connect(ui->illusionOffCheck, &QCheckBox::toggled, this, &WallTool::onFlagToggled);
    connect(ui->switchCheck, &QCheckBox::toggled, this, &WallTool::onFlagToggled);
    connect(ui->buddyProofCheck, &QCheckBox::toggled, this, &WallTool::onFlagToggled);
    connect(ui->ignoreMarkerCheck, &QCheckBox::toggled, this, &WallTool::onFlagToggled);
    
    // Action buttons
    connect(ui->addWallBtn, &QPushButton::clicked, this, &WallTool::onAddWall);
    connect(ui->deleteWallBtn, &QPushButton::clicked, this, &WallTool::onDeleteWall);
    connect(ui->otherSideBtn, &QPushButton::clicked, this, &WallTool::onOtherSide);
}

void WallTool::updateDisplay() {
    if (!m_mine) {
        return;
    }
    
    const Wall& wall = m_mine->getWall(m_currentWallId);
    
    // Block signals during update
    QSignalBlocker blocker1(ui->wallIdSpin);
    QSignalBlocker blocker2(ui->typeCombo);
    QSignalBlocker blocker3(ui->clipSpin);
    QSignalBlocker blocker4(ui->strengthSpin);
    QSignalBlocker blocker5(ui->cloakSpin);
    QSignalBlocker blocker6(ui->noKeyCheck);
    QSignalBlocker blocker7(ui->blueKeyCheck);
    QSignalBlocker blocker8(ui->redKeyCheck);
    QSignalBlocker blocker9(ui->goldKeyCheck);
    QSignalBlocker blocker10(ui->blastedCheck);
    QSignalBlocker blocker11(ui->doorOpenCheck);
    QSignalBlocker blocker12(ui->doorLockedCheck);
    QSignalBlocker blocker13(ui->doorAutoCheck);
    QSignalBlocker blocker14(ui->illusionOffCheck);
    QSignalBlocker blocker15(ui->switchCheck);
    QSignalBlocker blocker16(ui->buddyProofCheck);
    QSignalBlocker blocker17(ui->ignoreMarkerCheck);
    
    // Update wall ID
    ui->wallIdSpin->setValue(m_currentWallId);
    
    // Update type combo box (matches enum order)
    ui->typeCombo->setCurrentIndex(static_cast<int>(wall.getType()));
    
    // Update clip number
    ui->clipSpin->setValue(wall.getClipNum());
    
    // Update strength (convert from fixed point to percentage)
    // Assuming max strength is 100.0 in fixed point (6553600)
    double strengthPercent = (static_cast<double>(wall.getHitPoints()) / 65536.0);
    ui->strengthSpin->setValue(strengthPercent);
    
    // Update cloak value
    ui->cloakSpin->setValue(wall.getCloakValue());
    
    // Update key checkboxes
    uint8_t keys = wall.getKeys();
    ui->noKeyCheck->setChecked(keys == KeyNone);
    ui->blueKeyCheck->setChecked((keys & KeyBlue) != 0);
    ui->redKeyCheck->setChecked((keys & KeyRed) != 0);
    ui->goldKeyCheck->setChecked((keys & KeyGold) != 0);
    
    // Update flag checkboxes
    ui->blastedCheck->setChecked(wall.hasFlag(WallFlagBlasted));
    ui->doorOpenCheck->setChecked(wall.hasFlag(WallFlagDoorOpened));
    ui->doorLockedCheck->setChecked(wall.hasFlag(WallFlagDoorLocked));
    ui->doorAutoCheck->setChecked(wall.hasFlag(WallFlagDoorAuto));
    ui->illusionOffCheck->setChecked(wall.hasFlag(WallFlagIllusionOff));
    ui->switchCheck->setChecked(wall.hasFlag(WallFlagSwitchBlasted));
    ui->buddyProofCheck->setChecked(wall.hasFlag(WallFlagBuddyProof));
    ui->ignoreMarkerCheck->setChecked(wall.hasFlag(WallFlagIgnoreMarker));
}

void WallTool::enableControls(bool enable) {
    ui->wallDataGroup->setEnabled(enable);
    ui->keysGroup->setEnabled(enable);
    ui->flagsGroup->setEnabled(enable);
    ui->actionsGroup->setEnabled(enable);
}

void WallTool::onWallIdChanged(int value) {
    if (!m_mine || value < 0 || value >= m_mine->getWallCount()) {
        return;
    }
    
    m_currentWallId = value;
    updateDisplay();
}

void WallTool::onTypeChanged(int index) {
    if (!m_mine) {
        return;
    }
    
    WallType type = static_cast<WallType>(index);
    
    auto* mutableMine = const_cast<Mine*>(m_mine);
    mutableMine->getWall(m_currentWallId).setType(type);
    
    emit wallModified(m_currentWallId);
}

void WallTool::onClipChanged(int value) {
    if (!m_mine) {
        return;
    }
    
    auto* mutableMine = const_cast<Mine*>(m_mine);
    mutableMine->getWall(m_currentWallId).setClipNum(static_cast<int8_t>(value));
    
    emit wallModified(m_currentWallId);
}

void WallTool::onStrengthChanged(double value) {
    if (!m_mine) {
        return;
    }
    
    // Convert percentage to fixed point
    fix hitPoints = static_cast<fix>(value * 65536.0);
    
    auto* mutableMine = const_cast<Mine*>(m_mine);
    mutableMine->getWall(m_currentWallId).setHitPoints(hitPoints);
    
    emit wallModified(m_currentWallId);
}

void WallTool::onCloakChanged(int value) {
    if (!m_mine) {
        return;
    }
    
    auto* mutableMine = const_cast<Mine*>(m_mine);
    mutableMine->getWall(m_currentWallId).setCloakValue(static_cast<int8_t>(value));
    
    emit wallModified(m_currentWallId);
}

void WallTool::onKeyToggled(bool checked) {
    if (!m_mine) {
        return;
    }
    
    auto* sender = qobject_cast<QCheckBox*>(QObject::sender());
    if (!sender) {
        return;
    }
    
    auto* mutableMine = const_cast<Mine*>(m_mine);
    Wall& wall = mutableMine->getWall(m_currentWallId);
    
    uint8_t keys = wall.getKeys();
    
    // Handle "No Key" checkbox
    if (sender == ui->noKeyCheck) {
        if (checked) {
            keys = KeyNone;
        }
    } else {
        // Uncheck "No Key" if any other key is checked
        if (checked && sender != ui->noKeyCheck) {
            ui->noKeyCheck->setChecked(false);
        }
        
        // Update individual key bits
        if (sender == ui->blueKeyCheck) {
            if (checked) keys |= KeyBlue;
            else keys &= ~KeyBlue;
        } else if (sender == ui->redKeyCheck) {
            if (checked) keys |= KeyRed;
            else keys &= ~KeyRed;
        } else if (sender == ui->goldKeyCheck) {
            if (checked) keys |= KeyGold;
            else keys &= ~KeyGold;
        }
        
        // If no keys selected, check "No Key"
        if (keys == KeyNone) {
            ui->noKeyCheck->setChecked(true);
        }
    }
    
    wall.setKeys(keys);
    emit wallModified(m_currentWallId);
}

void WallTool::onFlagToggled(bool checked) {
    if (!m_mine) {
        return;
    }
    
    auto* sender = qobject_cast<QCheckBox*>(QObject::sender());
    if (!sender) {
        return;
    }
    
    auto* mutableMine = const_cast<Mine*>(m_mine);
    Wall& wall = mutableMine->getWall(m_currentWallId);
    
    // Determine which flag was toggled
    WallFlags flag = WallFlagBlasted; // Default
    
    if (sender == ui->blastedCheck) {
        flag = WallFlagBlasted;
    } else if (sender == ui->doorOpenCheck) {
        flag = WallFlagDoorOpened;
    } else if (sender == ui->doorLockedCheck) {
        flag = WallFlagDoorLocked;
    } else if (sender == ui->doorAutoCheck) {
        flag = WallFlagDoorAuto;
    } else if (sender == ui->illusionOffCheck) {
        flag = WallFlagIllusionOff;
    } else if (sender == ui->switchCheck) {
        flag = WallFlagSwitchBlasted;
    } else if (sender == ui->buddyProofCheck) {
        flag = WallFlagBuddyProof;
    } else if (sender == ui->ignoreMarkerCheck) {
        flag = WallFlagIgnoreMarker;
    }
    
    if (checked) {
        wall.addFlag(flag);
    } else {
        wall.removeFlag(flag);
    }
    
    emit wallModified(m_currentWallId);
}

void WallTool::onAddWall() {
    // TODO: Implement add wall functionality
    // Requires segment/side selection and wall creation
}

void WallTool::onDeleteWall() {
    // TODO: Implement delete wall functionality
    // Requires wall removal and segment side update
}

void WallTool::onOtherSide() {
    // TODO: Implement navigation to connected wall on other side
    // Requires segment connectivity lookup
}

} // namespace dle
