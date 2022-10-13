package java.lang;

public class Throwable {
    private String message;

    public Throwable(){
        message = null;
    }

    public Throwable(String details){
        message = details;
    }

    public String getMessage(){
        return message;
    }
}
