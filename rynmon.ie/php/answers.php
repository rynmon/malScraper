<?php
// For debugging (remove in production)
ini_set('display_errors', 1);
ini_set('display_startup_errors', 1);
error_reporting(E_ALL);

// Set appropriate headers
header('Content-Type: application/json');
header('Access-Control-Allow-Origin: *'); // Allow access from any domain (consider limiting this in production)
header('Access-Control-Allow-Methods: POST, OPTIONS');
header('Access-Control-Allow-Headers: Content-Type, X-Requested-With');

// Handle preflight OPTIONS request
if ($_SERVER['REQUEST_METHOD'] === 'OPTIONS') {
    http_response_code(200);
    exit;
}

// Check if this is a POST request
if ($_SERVER['REQUEST_METHOD'] !== 'POST') {
    http_response_code(405); // Method Not Allowed
    echo json_encode(['error' => 'Only POST requests are allowed']);
    exit;
}

// Temporarily disable the AJAX check for testing

// Answers are stored server-side and only verified answers are returned
// The client never sees the actual answers until after submission
$answers = [
    1 => 'luigi\'s mansion',
    2 => 'xcom',
    3 => 'hollow knight',
    4 => 'super mario sunshine',
    5 => 'dark souls',
    6 => 'metal gear solid',
    7 => 'counter strike',
    8 => 'factorio',
    9 => 'final fantasy vi',
    10 => 'diablo ii',
    11 => 'happy wheels',
    12 => 'rimworld',
    13 => 'death stranding',
    14 => 'portal 2',
    15 => 'terraria',
    16 => 'spelunky',
    17 => 'resident evil 2',
    18 => 'runescape',
    19 => 'minecraft',
    20 => 'fallout 3',
    21 => 'world of warcraft',
    22 => 'overwatch',
    23 => 'undertale',
    24 => 'fortnite'
];

// Check what action is being requested
$action = isset($_POST['action']) ? $_POST['action'] : '';

// For debugging
if (empty($action)) {
    $inputData = file_get_contents('php://input');
    if (!empty($inputData)) {
        $jsonData = json_decode($inputData, true);
        if ($jsonData && isset($jsonData['action'])) {
            $action = $jsonData['action'];
            $_POST = $jsonData;
        }
    }
}

if ($action === 'verify') {
    // Verify a specific answer
    $id = isset($_POST['id']) ? (int)$_POST['id'] : 0;
    $guess = isset($_POST['guess']) ? trim(strtolower($_POST['guess'])) : '';

    if ($id > 0 && isset($answers[$id])) {
        $correctAnswer = $answers[$id];

        // Perform the same normalization as on the client
        $normalizedGuess = preg_replace('/^the\s+/', '', $guess);
        $normalizedGuess = preg_replace('/[^\w\s]/', '', $normalizedGuess);
        $normalizedGuess = trim($normalizedGuess);

        $normalizedAnswer = preg_replace('/^the\s+/', '', $correctAnswer);
        $normalizedAnswer = preg_replace('/[^\w\s]/', '', $normalizedAnswer);
        $normalizedAnswer = trim($normalizedAnswer);

        // Check exact match
        $isCorrect = ($normalizedGuess === $normalizedAnswer);

        // Check main keywords if not an exact match
        if (!$isCorrect) {
            $guessWords = explode(' ', $normalizedGuess);
            $answerWords = explode(' ', $normalizedAnswer);

            $mainGuessWords = array_slice($guessWords, 0, min(count($guessWords), 3));
            $mainAnswerWords = array_slice($answerWords, 0, min(count($answerWords), 3));

            $isCorrect = (implode(' ', $mainGuessWords) === implode(' ', $mainAnswerWords));
        }

        // Only return the result, not the actual answer
        echo json_encode(['correct' => $isCorrect]);
    } else {
        echo json_encode(['error' => 'Invalid ID']);
    }
}
elseif ($action === 'results') {
    // This action returns the actual answers only after all guesses have been submitted
    $guesses = [];

    // Try to get guesses from POST data
    if (isset($_POST['guesses'])) {
        if (is_string($_POST['guesses'])) {
            $guesses = json_decode($_POST['guesses'], true);
        } else {
            $guesses = $_POST['guesses'];
        }
    }

    // If guesses still empty, try to get from raw input
    if (empty($guesses)) {
        $inputData = file_get_contents('php://input');
        if (!empty($inputData)) {
            $jsonData = json_decode($inputData, true);
            if ($jsonData && isset($jsonData['guesses'])) {
                $guesses = $jsonData['guesses'];
            }
        }
    }

    $results = [];
    $score = 0;

    // Verify all guesses and build results
    foreach ($guesses as $id => $guess) {
        $id = (int)$id;
        if ($id > 0 && isset($answers[$id])) {
            $correctAnswer = $answers[$id];

            // Normalize the guess and answer (guess is already lowercase from client)
            $normalizedGuess = preg_replace('/^the\s+/', '', trim($guess));
            $normalizedGuess = preg_replace('/[^\w\s]/', '', $normalizedGuess);
            $normalizedGuess = trim($normalizedGuess);

            $normalizedAnswer = preg_replace('/^the\s+/', '', $correctAnswer);
            $normalizedAnswer = preg_replace('/[^\w\s]/', '', $normalizedAnswer);
            $normalizedAnswer = trim($normalizedAnswer);

            // Check exact match
            $isCorrect = ($normalizedGuess === $normalizedAnswer);

            // Check main keywords if not an exact match
            if (!$isCorrect && !empty($normalizedGuess)) {
                $guessWords = explode(' ', $normalizedGuess);
                $answerWords = explode(' ', $normalizedAnswer);

                $mainGuessWords = array_slice($guessWords, 0, min(count($guessWords), 3));
                $mainAnswerWords = array_slice($answerWords, 0, min(count($answerWords), 3));

                $isCorrect = (implode(' ', $mainGuessWords) === implode(' ', $mainAnswerWords));
            }

            if ($isCorrect) {
                $score++;
            }

            $results[$id] = [
                'correct' => $isCorrect,
                'answer' => $answers[$id]  // Only now we return the actual answer
            ];
        }
    }

    echo json_encode([
        'results' => $results,
        'score' => $score
    ]);
}
elseif ($action === 'metadata') {
    // Return metadata about the sounds without any answers
    $metadata = [];
    foreach ($answers as $id => $answer) {
        $metadata[] = [
            'id' => $id,
            'file' => "sounds/sound{$id}.mp3"
        ];
    }
    echo json_encode(['sounds' => $metadata]);
}
else {
    // Log the received data for debugging
    $debugInfo = [
        'action' => $action,
        'post_data' => $_POST,
        'raw_input' => file_get_contents('php://input')
    ];

    echo json_encode([
        'error' => 'Invalid action',
        'debug' => $debugInfo  // Remove this in production
    ]);
}