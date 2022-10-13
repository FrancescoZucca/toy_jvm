package java.io;

public class FileDescriptor {

    private int fd;

    public FileDescriptor(int i){
        this.fd = i;
    }

    public static final FileDescriptor out = new FileDescriptor(1);
}
