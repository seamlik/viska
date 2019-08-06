package riko;

public class DoubleFreeException extends RuntimeException {
  public DoubleFreeException() {
    super("Attempting to free an HeapObject that is already freed!");
  }
}
