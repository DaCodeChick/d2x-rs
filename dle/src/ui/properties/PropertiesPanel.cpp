#include "PropertiesPanel.h"
#include "core/mine/Mine.h"
#include "core/mine/Segment.h"
#include "core/mine/Wall.h"
#include "core/mine/Trigger.h"
#include "core/mine/Object.h"
#include <format>

namespace dle {

PropertiesPanel::PropertiesPanel(QWidget* parent)
    : QWidget(parent)
{
    setupUI();
}

PropertiesPanel::~PropertiesPanel() = default;

void PropertiesPanel::setupUI() {
    auto* mainLayout = new QVBoxLayout(this);
    mainLayout->setContentsMargins(4, 4, 4, 4);
    mainLayout->setSpacing(4);

    // Title label
    m_titleLabel = new QLabel("Properties", this);
    QFont titleFont = m_titleLabel->font();
    titleFont.setBold(true);
    titleFont.setPointSize(titleFont.pointSize() + 1);
    m_titleLabel->setFont(titleFont);
    mainLayout->addWidget(m_titleLabel);

    // Scroll area for content
    m_scrollArea = new QScrollArea(this);
    m_scrollArea->setWidgetResizable(true);
    m_scrollArea->setFrameShape(QFrame::NoFrame);
    
    // Content widget inside scroll area
    m_contentWidget = new QWidget();
    m_contentLayout = new QVBoxLayout(m_contentWidget);
    m_contentLayout->setContentsMargins(0, 0, 0, 0);
    m_contentLayout->setSpacing(2);
    
    // Info label (main content display)
    m_infoLabel = new QLabel(this);
    m_infoLabel->setTextFormat(Qt::PlainText);
    m_infoLabel->setAlignment(Qt::AlignTop | Qt::AlignLeft);
    m_infoLabel->setWordWrap(true);
    QFont monoFont("Monospace", 9);
    monoFont.setStyleHint(QFont::Monospace);
    m_infoLabel->setFont(monoFont);
    
    m_contentLayout->addWidget(m_infoLabel);
    m_contentLayout->addStretch();
    
    m_scrollArea->setWidget(m_contentWidget);
    mainLayout->addWidget(m_scrollArea);

    showNoSelection();
}

void PropertiesPanel::setMine(const Mine* mine) {
    m_mine = mine;
    refresh();
}

void PropertiesPanel::refresh() {
    updateSelection();
}

void PropertiesPanel::updateSelection() {
    if (!m_mine) {
        showNoSelection();
        return;
    }

    // TODO: Get actual selection from editor state
    // For now, show placeholder that indicates selection type
    // This will be connected to selection system later
    showNoSelection();
}

void PropertiesPanel::clearDisplay() {
    m_infoLabel->clear();
}

void PropertiesPanel::showNoSelection() {
    m_titleLabel->setText("Properties");
    m_infoLabel->setText(
        "No selection\n\n"
        "Select a segment, wall,\n"
        "trigger, or object to\n"
        "view its properties."
    );
}

void PropertiesPanel::showSegmentProperties() {
    if (!m_mine) return;

    m_titleLabel->setText("Segment Properties");
    
    // TODO: Get current segment from selection
    // For now, show example format
    QString text;
    text += formatProperty("Segment ID", "0");
    text += formatProperty("Function", "None");
    text += formatProperty("Connected", "6 sides");
    text += "\n";
    text += formatProperty("Special", "Normal");
    text += formatProperty("Light", "100%");
    
    m_infoLabel->setText(text);
}

void PropertiesPanel::showWallProperties() {
    if (!m_mine) return;

    m_titleLabel->setText("Wall Properties");
    
    // TODO: Get current wall from selection
    QString text;
    text += formatProperty("Wall Type", "Normal");
    text += formatProperty("Clip #", "0");
    text += formatProperty("Keys", "None");
    text += "\n";
    text += formatProperty("Flags", "None");
    text += formatProperty("Strength", "100.0");
    
    m_infoLabel->setText(text);
}

void PropertiesPanel::showTriggerProperties() {
    if (!m_mine) return;

    m_titleLabel->setText("Trigger Properties");
    
    // TODO: Get current trigger from selection
    QString text;
    text += formatProperty("Trigger Type", "Open Door");
    text += formatProperty("Targets", "2");
    text += "\n";
    text += formatProperty("Flags", "None");
    text += formatProperty("Time", "0.0s");
    
    m_infoLabel->setText(text);
}

void PropertiesPanel::showObjectProperties() {
    if (!m_mine) return;

    m_titleLabel->setText("Object Properties");
    
    // TODO: Get current object from selection
    QString text;
    text += formatProperty("Object Type", "Robot");
    text += formatProperty("ID", "5");
    text += formatProperty("AI Mode", "Normal");
    text += "\n";
    text += formatProperty("Contains", "None");
    text += formatProperty("Count", "1");
    
    m_infoLabel->setText(text);
}

QString PropertiesPanel::formatProperty(const QString& label, const QString& value) {
    // Format: "Label: Value\n"
    // Pad label to 12 characters for alignment
    return std::format("{:<12}: {}\n", label.toStdString(), value.toStdString()).c_str();
}

} // namespace dle
