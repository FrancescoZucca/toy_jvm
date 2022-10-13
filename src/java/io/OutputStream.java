package java.io;

public abstract class OutputStream {

    public abstract void write(int b) throws NullPointerException, IndexOutOfBoundsException;

    public void write(byte[] b) throws NullPointerException, IndexOutOfBoundsException{
        write(b, 0, b.length);
    }

    private void write(byte[] b, int off, int len) throws NullPointerException, IndexOutOfBoundsException {
        if(b == null)
            throw new NullPointerException();
        else if ((off < 0) || (off > b.length) || (len < 0) || ((off + len) > b.length) || ((off + len) < 0))
            throw new IndexOutOfBoundsException();
        else if (len == 0)
            return;
        for (int i = 0 ; i < len ; i++) {
            write(b[off + i]);
        }
    }
}
