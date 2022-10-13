package java.lang;

public class String {
    private final char[] value;

    public String(){
        value = new char[0];
    }

    public String(char[] value){
        this.value = value;
    }

    public String(char[] buf, boolean b) {
        this.value = buf;
    }

    public int length(){
        return value.length;
    }

    public boolean isEmpty(){
        return value.length == 0;
    }

    public native byte[] getBytes();
    public static String valueOf(int i){
        return Integer.toString(i);
    }

    final static char[] digits = {
            0,1,2,3,4,5,6,7,8,9
    };
}
