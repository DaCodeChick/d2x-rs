#include "ObjectTool.h"
#include "ui_ObjectTool.h"
#include "../../core/mine/Mine.h"
#include "../../core/mine/Object.h"
#include <QSignalBlocker>

namespace dle {

ObjectTool::ObjectTool(QWidget* parent)
    : QWidget(parent)
    , ui(std::make_unique<Ui::ObjectTool>())
    , m_mine(nullptr)
    , m_updating(false)
{
    ui->setupUi(this);
    
    // Connect signals
    connect(ui->spinObjectNumber, QOverload<int>::of(&QSpinBox::valueChanged),
            this, &ObjectTool::onObjectNumberChanged);
    connect(ui->comboType, QOverload<int>::of(&QComboBox::currentIndexChanged),
            this, &ObjectTool::onTypeChanged);
    connect(ui->spinId, QOverload<int>::of(&QSpinBox::valueChanged),
            this, &ObjectTool::onIdChanged);
    
    connect(ui->spinSize, QOverload<double>::of(&QDoubleSpinBox::valueChanged),
            this, &ObjectTool::onSizeChanged);
    connect(ui->spinShields, QOverload<double>::of(&QDoubleSpinBox::valueChanged),
            this, &ObjectTool::onShieldsChanged);
    
    connect(ui->spinBehavior, QOverload<int>::of(&QSpinBox::valueChanged),
            this, &ObjectTool::onBehaviorChanged);
    connect(ui->spinHideSegment, QOverload<int>::of(&QSpinBox::valueChanged),
            this, &ObjectTool::onHideSegmentChanged);
    connect(ui->spinPathLength, QOverload<int>::of(&QSpinBox::valueChanged),
            this, &ObjectTool::onPathLengthChanged);
    
    connect(ui->spinCount, QOverload<int>::of(&QSpinBox::valueChanged),
            this, &ObjectTool::onCountChanged);
    
    connect(ui->comboContentsType, QOverload<int>::of(&QComboBox::currentIndexChanged),
            this, &ObjectTool::onContentsTypeChanged);
    connect(ui->spinContentsId, QOverload<int>::of(&QSpinBox::valueChanged),
            this, &ObjectTool::onContentsIdChanged);
    connect(ui->spinContentsCount, QOverload<int>::of(&QSpinBox::valueChanged),
            this, &ObjectTool::onContentsCountChanged);
    
    connect(ui->btnAdd, &QPushButton::clicked, this, &ObjectTool::onAddClicked);
    connect(ui->btnDelete, &QPushButton::clicked, this, &ObjectTool::onDeleteClicked);
    connect(ui->btnMove, &QPushButton::clicked, this, &ObjectTool::onMoveClicked);
    connect(ui->btnReset, &QPushButton::clicked, this, &ObjectTool::onResetClicked);
    connect(ui->btnDeleteAll, &QPushButton::clicked, this, &ObjectTool::onDeleteAllClicked);
    
    updateControlStates();
}

ObjectTool::~ObjectTool() = default;

void ObjectTool::setMine(const Mine* mine) {
    m_mine = mine;
    
    if (m_mine && m_mine->getObjectCount() > 0) {
        ui->spinObjectNumber->setMaximum(m_mine->getObjectCount() - 1);
        ui->spinObjectNumber->setValue(0);
    }
    
    updateControlStates();
    refresh();
}

void ObjectTool::refresh() {
    updateFromObject();
}

void ObjectTool::onObjectNumberChanged(int objectNum) {
    if (m_updating) return;
    updateFromObject();
}

void ObjectTool::onTypeChanged(int index) {
    if (m_updating) return;
    
    Object* obj = getCurrentObject();
    if (!obj) return;
    
    int8_t type = indexToObjectType(index);
    obj->setType(static_cast<ObjectType>(type));
    
    updateControlStates();
    emit objectModified();
}

void ObjectTool::onIdChanged(int id) {
    if (m_updating) return;
    
    Object* obj = getCurrentObject();
    if (!obj) return;
    
    obj->setId(static_cast<int8_t>(id));
    emit objectModified();
}

void ObjectTool::onSizeChanged(double size) {
    if (m_updating) return;
    
    Object* obj = getCurrentObject();
    if (!obj) return;
    
    obj->setSize(static_cast<fix>(size * 65536.0));  // Convert to fixed point
    emit objectModified();
}

void ObjectTool::onShieldsChanged(double shields) {
    if (m_updating) return;
    
    Object* obj = getCurrentObject();
    if (!obj) return;
    
    obj->setShields(static_cast<fix>(shields * 65536.0));  // Convert to fixed point
    emit objectModified();
}

void ObjectTool::onBehaviorChanged(int behavior) {
    if (m_updating) return;
    
    Object* obj = getCurrentObject();
    if (!obj || !obj->isRobot()) return;
    
    obj->getAIInfo().behavior = static_cast<uint8_t>(behavior);
    emit objectModified();
}

void ObjectTool::onHideSegmentChanged(int hideSegment) {
    if (m_updating) return;
    
    Object* obj = getCurrentObject();
    if (!obj || !obj->isRobot()) return;
    
    obj->getAIInfo().hideSegment = static_cast<int16_t>(hideSegment);
    emit objectModified();
}

void ObjectTool::onPathLengthChanged(int pathLength) {
    if (m_updating) return;
    
    Object* obj = getCurrentObject();
    if (!obj || !obj->isRobot()) return;
    
    obj->getAIInfo().pathLength = static_cast<int16_t>(pathLength);
    emit objectModified();
}

void ObjectTool::onCountChanged(int count) {
    if (m_updating) return;
    
    Object* obj = getCurrentObject();
    if (!obj || !obj->isPowerup()) return;
    
    obj->getPowerupInfo().count = static_cast<int32_t>(count);
    emit objectModified();
}

void ObjectTool::onContentsTypeChanged(int index) {
    if (m_updating) return;
    
    Object* obj = getCurrentObject();
    if (!obj) return;
    
    ObjectContents contents = obj->getContents();
    contents.type = indexToContentsType(index);
    obj->setContents(contents);
    
    updateControlStates();
    emit objectModified();
}

void ObjectTool::onContentsIdChanged(int id) {
    if (m_updating) return;
    
    Object* obj = getCurrentObject();
    if (!obj) return;
    
    ObjectContents contents = obj->getContents();
    contents.id = static_cast<int8_t>(id);
    obj->setContents(contents);
    emit objectModified();
}

void ObjectTool::onContentsCountChanged(int count) {
    if (m_updating) return;
    
    Object* obj = getCurrentObject();
    if (!obj) return;
    
    ObjectContents contents = obj->getContents();
    contents.count = static_cast<int8_t>(count);
    obj->setContents(contents);
    emit objectModified();
}

void ObjectTool::onAddClicked() {
    // TODO: Implement add object functionality
    // This would require Mine to be mutable and have addObject() method
}

void ObjectTool::onDeleteClicked() {
    // TODO: Implement delete object functionality
    // This would require Mine to be mutable and have deleteObject() method
}

void ObjectTool::onMoveClicked() {
    // TODO: Implement move object functionality
    // This would enter a mode to move the object to a new position
}

void ObjectTool::onResetClicked() {
    // TODO: Implement reset object functionality
    // This would reset object to default values for its type
}

void ObjectTool::onDeleteAllClicked() {
    // TODO: Implement delete all objects functionality
    // This would require confirmation and Mine to be mutable
}

void ObjectTool::updateFromObject() {
    if (!m_mine) {
        return;
    }
    
    const Object* obj = getCurrentObject();
    if (!obj) {
        return;
    }
    
    m_updating = true;
    
    // Block all signals during update
    QSignalBlocker blockers[] = {
        QSignalBlocker(ui->comboType),
        QSignalBlocker(ui->spinId),
        QSignalBlocker(ui->spinSegment),
        QSignalBlocker(ui->spinSize),
        QSignalBlocker(ui->spinShields),
        QSignalBlocker(ui->spinBehavior),
        QSignalBlocker(ui->spinHideSegment),
        QSignalBlocker(ui->spinPathLength),
        QSignalBlocker(ui->spinCount),
        QSignalBlocker(ui->comboContentsType),
        QSignalBlocker(ui->spinContentsId),
        QSignalBlocker(ui->spinContentsCount)
    };
    
    // Update basic info
    ui->comboType->setCurrentIndex(objectTypeToIndex(static_cast<int8_t>(obj->getType())));
    ui->spinId->setValue(obj->getId());
    ui->spinSegment->setValue(obj->getSegment());
    
    // Update properties (convert from fixed point)
    ui->spinSize->setValue(static_cast<double>(obj->getSize()) / 65536.0);
    ui->spinShields->setValue(static_cast<double>(obj->getShields()) / 65536.0);
    
    // Update AI settings
    const AIInfo& aiInfo = obj->getAIInfo();
    ui->spinBehavior->setValue(aiInfo.behavior);
    ui->spinHideSegment->setValue(aiInfo.hideSegment);
    ui->spinPathLength->setValue(aiInfo.pathLength);
    
    // Update powerup settings
    const PowerupInfo& powerupInfo = obj->getPowerupInfo();
    ui->spinCount->setValue(powerupInfo.count);
    
    // Update contents
    const ObjectContents& contents = obj->getContents();
    ui->comboContentsType->setCurrentIndex(contentsTypeToIndex(contents.type));
    ui->spinContentsId->setValue(contents.id);
    ui->spinContentsCount->setValue(contents.count);
    
    m_updating = false;
    
    updateControlStates();
}

Object* ObjectTool::getCurrentObject() {
    if (!m_mine) {
        return nullptr;
    }
    
    int objectNum = ui->spinObjectNumber->value();
    if (objectNum < 0 || objectNum >= m_mine->getObjectCount()) {
        return nullptr;
    }
    
    // Cast away const - we need mutable access for editing
    return const_cast<Object*>(&m_mine->getObject(objectNum));
}

const Object* ObjectTool::getCurrentObject() const {
    if (!m_mine) {
        return nullptr;
    }
    
    int objectNum = ui->spinObjectNumber->value();
    if (objectNum < 0 || objectNum >= m_mine->getObjectCount()) {
        return nullptr;
    }
    
    return &m_mine->getObject(objectNum);
}

void ObjectTool::updateControlStates() {
    const Object* obj = getCurrentObject();
    bool hasObject = (obj != nullptr);
    
    // Enable/disable based on whether we have an object
    ui->objectSelectionGroup->setEnabled(hasObject);
    ui->locationGroup->setEnabled(hasObject);
    ui->propertiesGroup->setEnabled(hasObject);
    ui->actionsGroup->setEnabled(hasObject);
    
    if (!obj) {
        ui->aiGroup->setEnabled(false);
        ui->powerupGroup->setEnabled(false);
        ui->contentsGroup->setEnabled(false);
        return;
    }
    
    // Enable AI group only for robots
    ui->aiGroup->setEnabled(obj->isRobot());
    
    // Enable powerup group only for powerups
    ui->powerupGroup->setEnabled(obj->isPowerup());
    
    // Enable contents group (for robots, but allow for others too)
    ui->contentsGroup->setEnabled(true);
    
    // Enable/disable contents ID and count based on type
    int contentsTypeIndex = ui->comboContentsType->currentIndex();
    bool hasContents = (contentsTypeIndex > 0);  // 0 is "None"
    ui->spinContentsId->setEnabled(hasContents);
    ui->spinContentsCount->setEnabled(hasContents);
}

int ObjectTool::objectTypeToIndex(int8_t type) const {
    // Map ObjectType enum to combo box index
    // Combo items: Robot, Hostage, Player, Weapon, Powerup, Reactor, Coop, Cambot, Monsterball, Smoke, Explosion, Effect
    // Enum values: Robot=2, Hostage=3, Player=4, Weapon=5, Powerup=7, Reactor=9, Coop=14, Cambot=16, Monsterball=17, Smoke=18, Explosion=19, Effect=20
    switch (type) {
        case 2:  return 0;  // Robot
        case 3:  return 1;  // Hostage
        case 4:  return 2;  // Player
        case 5:  return 3;  // Weapon
        case 7:  return 4;  // Powerup
        case 9:  return 5;  // Reactor
        case 14: return 6;  // Coop
        case 16: return 7;  // Cambot
        case 17: return 8;  // Monsterball
        case 18: return 9;  // Smoke
        case 19: return 10; // Explosion
        case 20: return 11; // Effect
        default: return 0;
    }
}

int8_t ObjectTool::indexToObjectType(int index) const {
    // Map combo box index to ObjectType enum value
    switch (index) {
        case 0:  return 2;  // Robot
        case 1:  return 3;  // Hostage
        case 2:  return 4;  // Player
        case 3:  return 5;  // Weapon
        case 4:  return 7;  // Powerup
        case 5:  return 9;  // Reactor
        case 6:  return 14; // Coop
        case 7:  return 16; // Cambot
        case 8:  return 17; // Monsterball
        case 9:  return 18; // Smoke
        case 10: return 19; // Explosion
        case 11: return 20; // Effect
        default: return 2;
    }
}

int ObjectTool::contentsTypeToIndex(int8_t type) const {
    // Map contents type to combo box index
    // Combo items: None, Robot, Powerup
    // Enum values: Robot=2, Powerup=7
    switch (type) {
        case 2:  return 1;  // Robot
        case 7:  return 2;  // Powerup
        default: return 0;  // None
    }
}

int8_t ObjectTool::indexToContentsType(int index) const {
    // Map combo box index to contents type
    switch (index) {
        case 1:  return 2;  // Robot
        case 2:  return 7;  // Powerup
        default: return -1; // None
    }
}

} // namespace dle
