package riko;

public class UseAfterFreeException extends RuntimeException {
  public UseAfterFreeException() {
    super("Attempting to use a HeapObject after it is freed!");
  }
}
