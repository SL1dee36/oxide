CC ?= gcc
CFLAGS ?= -std=c99 -Wall -Wextra -O2
OXIDE_ROOT = ../..
INCLUDES = -I$(OXIDE_ROOT)/include

ifeq ($(OS),Windows_NT)
    TARGET = c_app.exe
    LDFLAGS = -L$(OXIDE_ROOT)/target/release -loxide
else
    UNAME_S := $(shell uname -s)
    TARGET = c_app
    ifeq ($(UNAME_S),Darwin)
        LDFLAGS = -L$(OXIDE_ROOT)/target/release -loxide -Wl,-rpath,$(OXIDE_ROOT)/target/release
    else
        LDFLAGS = -L$(OXIDE_ROOT)/target/release -loxide -Wl,-rpath,'$$ORIGIN/../../target/release' -lm -ldl -lpthread
    endif
endif

.PHONY: all clean

all: $(TARGET)

$(TARGET): main.c
	$(CC) $(CFLAGS) $(INCLUDES) main.c $(LDFLAGS) -o $(TARGET)

clean:
	rm -f $(TARGET) *.o
