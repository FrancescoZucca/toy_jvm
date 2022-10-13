package java.io;

public class FileOutputStream extends OutputStream{

    private FileDescriptor fd;

    public FileOutputStream(FileDescriptor fd){
        this.fd = fd;
    }

    public native void write(int b) throws NullPointerException, RuntimeException;
}
