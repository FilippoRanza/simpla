#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define TRUE 1
#define FALSE 0
#define BUFF_SIZE 1024

#define STR_STATUS_ALLOC 0
#define STR_STATUS_STATIC 1
#define STR_STATUS_REF 2

char* _INPUT_BUFFER = NULL;

typedef struct {
    char* buffer;
    size_t len;
    int status;
} _simpla_string;

void _free_simpla_string(_simpla_string* str) {
    if(str->status == STR_STATUS_ALLOC && str->buffer != NULL) {
        free(str->buffer);
    }
    free(str);
}

typedef struct _string_node {
    size_t context;
    _simpla_string* data;
    struct _string_node* next;
} _string_node;

typedef struct {
    _string_node* collection;
    size_t current_context;
} _string_collector;



_string_collector* _init_context() {
    _string_collector* output = malloc(sizeof(_string_collector));
    if (output == NULL) {
        abort();
    }
    output->collection = NULL;
    output->current_context = 0;

    return output;
}


void _next_context(_string_collector* str_col) {
    str_col->current_context++;
}

void _add_simpla_string(_string_collector* str_col, _simpla_string* str) {
    _string_node* node = malloc(sizeof(_string_node));
    if(node == NULL) {
        abort();
    }

    node->context = str_col->current_context;
    node->data = str;
    node->next = str_col->collection;
    str_col->collection = node;

}

void _clear_context(_string_collector* str_col) {
    while(str_col->collection != NULL && str_col->collection->context == str_col->current_context) {
        _string_node* tmp = str_col->collection;
        _free_simpla_string(tmp->data);
        str_col->collection = tmp->next;
        free(tmp);
    }
    if(str_col->current_context) {
        str_col->current_context--;
    }
}


_simpla_string* _alloc_simpla_string(_string_collector* str_col, size_t len) {
    _simpla_string* output = malloc(sizeof(_simpla_string));
    if(output == NULL) {
        abort();
    }

    if(len) {
        len++;
        char* buffer = malloc(sizeof(char) * len);
        if(buffer == NULL) {
            abort();
        }
        output->buffer = buffer;
        output->len = len;
        while(len--)
            *buffer++ = '\0';
            
    } else {
        output->buffer = NULL;
        output->len = 0;
    }
    output->status = STR_STATUS_ALLOC;

    _add_simpla_string(str_col, output);

    return output;
}


_simpla_string* _copy_simpla_string(_string_collector* str_col, _simpla_string* str) {
    _simpla_string* output = _alloc_simpla_string(str_col, 0);

    output->buffer = str->buffer;
    output->len = str->len;
    output->status = STR_STATUS_REF;

    return output;
}

_simpla_string* _clone_simpla_string(_string_collector* str_col, _simpla_string* str) {
    _simpla_string* output = _alloc_simpla_string(str_col, str->len);
    strncpy(output->buffer, str->buffer, str->len);
    return output;
}

_simpla_string* _from_static(_string_collector* str_col, char* static_str) {
    _simpla_string* output = _alloc_simpla_string(str_col, 0);
    output->buffer = static_str;
    output->len = strlen(static_str);
    output->status = STR_STATUS_STATIC;
    return output;
}

_string_collector* _GLOBAL_COLLECTOR = NULL;

char* _alloc_buffer() {

    char* output = calloc(BUFF_SIZE, sizeof(char));
    if(output == NULL) {
        fprintf(stderr, "cannot allocate buffer of size: %d", BUFF_SIZE);
        abort();
    }

    return output;
}


void _read_buffer(char* buff) {
    char* tmp = buff;
    int c;
    int count = BUFF_SIZE - 1;
    while((c = getchar()) && c != EOF && c != '\n' && count--) 
        *tmp++ = c;
    *tmp = '\0';
}


char _read_bool() {
    _read_buffer(_INPUT_BUFFER);
    int tmp = atoi(_INPUT_BUFFER);
    return tmp ? TRUE : FALSE;
}

int _read_int() {
    _read_buffer(_INPUT_BUFFER);
    return atoi(_INPUT_BUFFER);
}

double _read_double() {
    _read_buffer(_INPUT_BUFFER);
    return atof(_INPUT_BUFFER);
}

char* _read_str(char* str) {
    if(str == NULL) {
        str = _alloc_buffer();
    }
    _read_buffer(str);
    return str;
}

void _initialize() {
    _INPUT_BUFFER = _alloc_buffer();
    _GLOBAL_COLLECTOR = _init_context();
}

void _finalize() {
    free(_INPUT_BUFFER);
    _clear_context(_GLOBAL_COLLECTOR);
}

void _free_str(char* str) {
    if(str != NULL) {
        free(str);
    }
}
