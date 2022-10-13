JAVA_SOURCES = $(wildcard */*/*/*.java)
SOURCES = $(wildcard *.java)

all:
	javac --boot-class-path src/ --source 8 --target 8 $(JAVA_SOURCES)
	javac --boot-class-path src/ --source 8 --target 8 $(SOURCES)
