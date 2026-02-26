#include <QApplication>
#include <QMainWindow>
#include <QLabel>
#include "core/mine/Mine.h"

int main(int argc, char *argv[]) {
    QApplication app(argc, argv);
    
    // Test creating a mine
    dle::Mine mine;
    mine.createDefault();
    
    // Create a simple window to verify Qt works
    QMainWindow window;
    window.setWindowTitle("DLE - Descent Level Editor");
    window.resize(800, 600);
    
    QLabel* label = new QLabel(QString("DLE - Descent Level Editor\n\nMine has %1 segments and %2 vertices")
                                   .arg(mine.getSegmentCount())
                                   .arg(mine.getVertexCount()), 
                               &window);
    label->setAlignment(Qt::AlignCenter);
    window.setCentralWidget(label);
    
    window.show();
    
    return app.exec();
}
