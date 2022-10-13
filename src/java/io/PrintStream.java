package java.io;

public class PrintStream extends OutputStream{

    private OutputStream out;

    public PrintStream(OutputStream out){
        this.out = out;
    }

    public void write(int b) throws IndexOutOfBoundsException, NullPointerException {
        out.write(b);
    }

    public void print(int i) throws IndexOutOfBoundsException, NullPointerException{
        out.write(String.valueOf(i).getBytes());
    }

    public void println(int i) throws NullPointerException, IndexOutOfBoundsException{
        print(i);
        out.write("\n".getBytes());
    }
}
