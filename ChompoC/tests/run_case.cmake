execute_process(
        COMMAND
        "${CHOMPO_EXECUTABLE}"
        "${SOURCE_FILE}"
        RESULT_VARIABLE actual_exit
        OUTPUT_VARIABLE actual_stdout
        ERROR_VARIABLE actual_stderr
)

file(READ
        "${EXPECTED_FILE}"
        expected_output)

if(NOT actual_exit EQUAL EXPECTED_EXIT_CODE)
    message(FATAL_ERROR
            "Wrong exit code.\n"
            "Expected: ${EXPECTED_EXIT_CODE}\n"
            "Actual: ${actual_exit}\n"
            "stdout:\n${actual_stdout}\n"
            "stderr:\n${actual_stderr}"
    )
endif()

if(USE_STDERR)
    set(actual_output "${actual_stderr}")
else()
    set(actual_output "${actual_stdout}")
endif()

string(REPLACE "\r\n" "\n"
        actual_output
        "${actual_output}"
)

string(REPLACE "\r\n" "\n"
        expected_output
        "${expected_output}"
)

string(REGEX REPLACE "[\r\n]+$" ""
        actual_output
        "${actual_output}"
)

string(REGEX REPLACE "[\r\n]+$" ""
        expected_output
        "${expected_output}"
)

string(REPLACE "\r\n" "\n"
        actual_output
        "${actual_output}"
)

string(REPLACE "\r\n" "\n"
        expected_output
        "${expected_output}"
)

if(USE_STDERR)
    string(FIND
            "${actual_output}"
            "${expected_output}"
            match_position
    )

    if(match_position EQUAL -1)
        message(FATAL_ERROR
                "Expected error fragment was not found.\n"
                "Expected fragment:\n${expected_output}\n"
                "Actual stderr:\n${actual_output}"
        )
    endif()
elseif(NOT actual_output STREQUAL expected_output)
    message(FATAL_ERROR
            "Output mismatch.\n"
            "Expected:\n${expected_output}\n"
            "Actual:\n${actual_output}"
    )
endif()