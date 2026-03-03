#include <QApplication>
#include <QIcon>
#include "ui/mainwindow/MainWindow.h"

int main(int argc, char *argv[]) {
    QApplication app(argc, argv);
    
    // Set application info
    app.setApplicationName("DLE");
    app.setApplicationDisplayName("Descent Level Editor");
    app.setApplicationVersion("0.1.0");
    app.setOrganizationName("Descent Community");
    
    // Set application icon
    app.setWindowIcon(QIcon(":/icons/dlexp.ico"));
    
    dle::MainWindow window;
    window.show();
    
    return app.exec();
}
