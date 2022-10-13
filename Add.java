public class Add{

	private int test = 5;
	public static int Add(int a, int b){
		return a + b;
	}

	public static double Add(double a, double b){return a + b;}

	public static int Sub(int a, int b){
		return a - b;
	}

	public int add_test(int a){
		return a + test;
	}

	public static void main(String[] args){
		Add(3, 4);
		Sub(5, 3);
		Add(5.5d, 4.3d);

		int[] ints = new int[3];

		Add add = new Add();
		add.add_test(6);
	}
}
