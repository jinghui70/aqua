/**
 * MySQL resolver — maps MySQL JDBC raw physical types to aqua logical DataType.
 */

module.exports = {
  type: 'mysql',

  resolve: function (column) {
    var raw = column.rawType.toUpperCase();
    var length = column.length;
    var precision = column.precision;
    var scale = column.scale;

    // VARCHAR / CHAR → VARCHAR(n)
    if (raw === 'VARCHAR' || raw === 'CHAR') {
      return { dataType: 'VARCHAR', length: length > 0 ? length : undefined };
    }

    // TEXT / MEDIUMTEXT / LONGTEXT / TINYTEXT → CLOB
    if (raw === 'TEXT' || raw === 'MEDIUMTEXT' || raw === 'LONGTEXT' || raw === 'TINYTEXT') {
      return { dataType: 'CLOB' };
    }

    // TINYINT → TINYINT
    if (raw === 'TINYINT') {
      return { dataType: 'TINYINT' };
    }

    // INT / INTEGER / MEDIUMINT → INT
    if (raw === 'INT' || raw === 'INTEGER' || raw === 'MEDIUMINT') {
      return { dataType: 'INT' };
    }

    // BIGINT → LONG
    if (raw === 'BIGINT') {
      return { dataType: 'LONG' };
    }

    // DECIMAL / NUMERIC → DECIMAL(p,s)
    if (raw === 'DECIMAL' || raw === 'NUMERIC') {
      var p = precision != null && precision > 0 ? precision : undefined;
      var s = scale != null && scale > 0 ? scale : undefined;
      return { dataType: 'DECIMAL', precision: p, scale: s };
    }

    // DATE → DATE
    if (raw === 'DATE') {
      return { dataType: 'DATE' };
    }

    // DATETIME / TIMESTAMP → DATETIME
    if (raw === 'DATETIME' || raw === 'TIMESTAMP') {
      return { dataType: 'DATETIME' };
    }

    // BLOB family
    if (raw === 'BLOB' || raw === 'TINYBLOB' || raw === 'MEDIUMBLOB' || raw === 'LONGBLOB' || raw === 'VARBINARY' || raw === 'BINARY') {
      return { dataType: 'BLOB' };
    }

    // FLOAT / DOUBLE → skip (not in 9 aqua types, return null)
    return null;
  }
};
