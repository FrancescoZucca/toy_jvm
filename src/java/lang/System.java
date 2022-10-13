package java.lang;

import java.io.FileDescriptor;
import java.io.FileOutputStream;
import java.io.PrintStream;

public class System {

    private static native void registerNatives();
    static {
        registerNatives();
    }

    public final static PrintStream out = new PrintStream(new FileOutputStream(FileDescriptor.out));


}
